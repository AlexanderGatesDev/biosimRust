// spawn_new_generation.rs
// Spawns new generations from parent genomes

use crate::grid::Grid;
use crate::signals::Signals;
use crate::peeps::Peeps;
use crate::genome_neurons::{Genome, make_random_genome, generate_child_genome};
use crate::params::Params;
use crate::survival_criteria::passed_survival_criterion;
use crate::simulator::*;
use crate::random::random_uint_range;
use crate::create_barrier::create_barrier;
use crate::metabolic_cost::{reset_metabolic_tracking, reward_energy};
use crate::homeostasis::reset_homeostasis;
use crate::multi_objective::{FitnessObjectives, IndividualWithObjectives, nsga2_selection};
use crate::speciation::{update_species_stats, cull_stagnant_species, limit_species_size, update_species_representatives, get_species_list, reset_species_list, assign_survivors_to_species, select_parents_with_speciation, get_next_species_id, assign_to_species};
use std::collections::HashMap;

pub fn initialize_generation_0(
    grid: &mut Grid,
    signals: &mut Signals,
    peeps: &mut Peeps,
    params: &Params,
) {
    grid.zero_fill();
    create_barrier(grid, params.barrier_type, params);
    signals.zero_fill();

    for index in 1..=params.population {
        let loc = grid.find_empty_location(params);
        let genome = make_random_genome(params);
        
        if let Some(indiv) = peeps.get_mut(index as u16) {
            indiv.initialize(index as u16, loc, genome, params);
            grid.set(loc, index as u16);
        }
    }
}

pub fn initialize_new_generation(
    parent_genomes: &[Genome],
    _generation: u32,
    grid: &mut Grid,
    signals: &mut Signals,
    peeps: &mut Peeps,
    params: &Params,
) {
    grid.zero_fill();
    create_barrier(grid, params.barrier_type, params);
    signals.zero_fill();

    for index in 1..=params.population {
        let loc = grid.find_empty_location(params);
        let genome = generate_child_genome(parent_genomes, params);
        
        if let Some(indiv) = peeps.get_mut(index as u16) {
            indiv.initialize(index as u16, loc, genome, params);
            grid.set(loc, index as u16);
        }
    }
}

pub fn spawn_new_generation(
    generation: u32,
    murder_count: u32,
    grid: &mut Grid,
    signals: &mut Signals,
    peeps: &mut Peeps,
    params: &Params,
) -> u32 {
    let mut sacrificed_count = 0u32;

    // Phase 6: Initialize species list at generation 0
    if generation == 0 && params.speciation_enabled {
        reset_species_list();
    }

    let mut parents: Vec<(u16, f32)> = Vec::new(); // (indiv index, score)
    let mut parent_genomes: Vec<Genome> = Vec::new();

    // Phase 5: Use NSGA-II multi-objective selection if enabled
    if params.nsga2_enabled && params.challenge != CHALLENGE_ALTRUISM {
        // Collect candidates with multi-objective fitness
        let mut candidates: Vec<IndividualWithObjectives> = Vec::new();
        
        for index in 1..=params.population {
            if let Some(indiv) = peeps.get(index as u16) {
                let (passed, score) = passed_survival_criterion(indiv, params.challenge, params, grid);
                if passed && !indiv.nnet.connections.is_empty() {
                    let objectives = FitnessObjectives::from_indiv(
                        index as u16,
                        score,
                        indiv.metabolic_cost_accumulated,
                        params.steps_per_generation,
                        peeps,
                        params,
                    );
                    
                    candidates.push(IndividualWithObjectives {
                        index: index as u16,
                        objectives,
                        rank: 0,
                        crowding_distance: 0.0,
                    });
                }
            }
        }
        
        if !candidates.is_empty() {
            // Phase 6: Apply speciation if enabled
            if params.speciation_enabled {
                // Collect survivors for species assignment
                let survivors: Vec<(u16, f32)> = candidates.iter()
                    .map(|c| (c.index, c.objectives.survival_score))
                    .collect();
                
                // Assign survivors to species
                let species_assignments = assign_survivors_to_species(&survivors, peeps, params);
                
                // Get species list for selection
                let mut list_guard = get_species_list();
                let species_list = list_guard.as_mut().unwrap();
                
                // Update species statistics
                for species in species_list.iter_mut() {
                    if !species.members.is_empty() {
                        let best_fitness = species.members.iter()
                            .filter_map(|&idx| {
                                candidates.iter().find(|c| c.index == idx)
                                    .map(|c| c.objectives.survival_score)
                            })
                            .fold(0.0f32, f32::max);
                        update_species_stats(species, generation, best_fitness);
                    }
                }
                
                // Use speciation-based selection
                let num_parents = candidates.len().min(params.population as usize);
                let selected_indices = select_parents_with_speciation(
                    &survivors,
                    &species_assignments,
                    species_list,
                    num_parents,
                    params,
                );
                
                // Convert to parent list format
                for idx in selected_indices {
                    if let Some(candidate) = candidates.iter().find(|c| c.index == idx) {
                        parents.push((candidate.index, candidate.objectives.survival_score));
                    }
                }
            } else {
                // Use NSGA-II selection without speciation
                let num_parents = candidates.len().min(params.population as usize);
                let selected_indices = nsga2_selection(&mut candidates, num_parents);
                
                // Convert to parent list format
                for idx in selected_indices {
                    parents.push((candidates[idx].index, candidates[idx].objectives.survival_score));
                }
            }
        }
    } else if params.challenge != CHALLENGE_ALTRUISM {
        // Original single-objective selection
        let mut survivors: Vec<(u16, f32)> = Vec::new();
        
        for index in 1..=params.population {
            if let Some(indiv) = peeps.get(index as u16) {
                let (passed, score) = passed_survival_criterion(indiv, params.challenge, params, grid);
                if passed && !indiv.nnet.connections.is_empty() {
                    // Use accumulated metabolic cost for the entire generation (Phase 2)
                    // The accumulated cost is the total over all steps
                    // Scale it to be comparable to fitness scores (0.0-1.0 range)
                    // Cost per step is small (0.001-0.01), so total over 300 steps is 0.3-3.0
                    // We normalize by steps to get average cost per step, which is comparable to fitness
                    let steps_per_generation = params.steps_per_generation as f32;
                    let metabolic_cost = (indiv.metabolic_cost_accumulated / steps_per_generation).min(score);
                    
                    // Energy-based fitness: reward minus scaled metabolic cost
                    // Cap cost at score to ensure net_fitness >= 0
                    let net_fitness = score - metabolic_cost;
                    
                    // Apply minimal length normalization (for backward compatibility)
                    let genome_length = indiv.genome.len() as f32;
                    let normalized_score = net_fitness / (1.0 + params.fitness_length_normalization * genome_length);
                    
                    // Only include individuals with positive fitness
                    if normalized_score > 0.0 {
                        survivors.push((index as u16, normalized_score));
                    }
                }
            }
        }
        
        // Phase 6: Apply speciation if enabled
        if params.speciation_enabled && !survivors.is_empty() {
            // Assign survivors to species
            let species_assignments = assign_survivors_to_species(&survivors, peeps, params);
            
            // Get species list for selection and stats update
            let mut list_guard = get_species_list();
            let species_list = list_guard.as_mut().unwrap();
            
            // Update species statistics
            for species in species_list.iter_mut() {
                if !species.members.is_empty() {
                    let best_fitness = species.members.iter()
                        .filter_map(|&idx| {
                            survivors.iter().find(|(i, _)| *i == idx)
                                .map(|(_, fitness)| *fitness)
                        })
                        .fold(0.0f32, f32::max);
                    update_species_stats(species, generation, best_fitness);
                }
            }
            
            // Use speciation-based selection
            let num_parents = survivors.len().min(params.population as usize);
            let selected_indices = select_parents_with_speciation(
                &survivors,
                &species_assignments,
                species_list,
                num_parents,
                params,
            );
            
            // Convert to parent list
            for idx in selected_indices {
                if let Some(&(parent_idx, fitness)) = survivors.iter().find(|(i, _)| *i == idx) {
                    parents.push((parent_idx, fitness));
                }
            }
        } else {
            // No speciation - use original selection
            parents = survivors;
        }
    } else {
        // Altruism challenge
        let consider_kinship = true;
        let mut sacrifice_indexes: Vec<u16> = Vec::new();

        for index in 1..=params.population {
            if let Some(indiv) = peeps.get(index as u16) {
                let (passed, score) = passed_survival_criterion(indiv, CHALLENGE_ALTRUISM, params, grid);
                if passed && !indiv.nnet.connections.is_empty() {
                    // Use accumulated metabolic cost, scaled to fitness range
                    let steps_per_generation = params.steps_per_generation as f32;
                    let metabolic_cost = indiv.metabolic_cost_accumulated / steps_per_generation;
                    let net_fitness = score - metabolic_cost;
                    
                    // Apply minimal length normalization
                    let genome_length = indiv.genome.len() as f32;
                    let normalized_score = net_fitness / (1.0 + params.fitness_length_normalization * genome_length);
                    
                    if normalized_score > 0.0 {
                        parents.push((index as u16, normalized_score));
                    }
                } else {
                    let (passed_sacrifice, _) = passed_survival_criterion(indiv, CHALLENGE_ALTRUISM_SACRIFICE, params, grid);
                    if passed_sacrifice && !indiv.nnet.connections.is_empty() {
                        if consider_kinship {
                            sacrifice_indexes.push(index as u16);
                        } else {
                            sacrificed_count += 1;
                        }
                    }
                }
            }
        }

        let generation_to_apply_kinship = 10u32;
        const ALTRUISM_FACTOR: u32 = 10;

        if consider_kinship && generation > generation_to_apply_kinship {
            let threshold = 0.7f32;
            let mut surviving_kin: Vec<(u16, f32)> = Vec::new();

            for _passes in 0..ALTRUISM_FACTOR {
                for &sacrificed_index in &sacrifice_indexes {
                    let start_index = random_uint_range(0, parents.len() as u32 - 1) as usize;
                    for count in 0..parents.len() {
                        let possible_parent = &parents[(start_index + count) % parents.len()];
                        if let (Some(g1), Some(g2)) = (
                            peeps.get(sacrificed_index).map(|i| &i.genome),
                            peeps.get(possible_parent.0).map(|i| &i.genome),
                        ) {
                            let similarity = crate::genome_compare::genome_similarity(g1, g2, params);
                            if similarity >= threshold {
                                surviving_kin.push(*possible_parent);
                                break;
                            }
                        }
                    }
                }
            }
            println!("{} passed, {} sacrificed, {} saved", parents.len(), sacrifice_indexes.len(), surviving_kin.len());
            parents = surviving_kin;
        } else {
            let number_saved = sacrificed_count * ALTRUISM_FACTOR;
            println!("{} passed, {} sacrificed, {} saved", parents.len(), sacrificed_count, number_saved);
            if !parents.is_empty() && number_saved < parents.len() as u32 {
                parents.truncate(number_saved as usize);
            }
        }
    }

    // Sort parents by score (descending) - only if not using NSGA-II or speciation (already sorted)
    if !params.nsga2_enabled && !params.speciation_enabled {
        parents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    }
    
    // Phase 6: Post-selection species management
    if params.speciation_enabled {
        let mut list_guard = get_species_list();
        let species_list = list_guard.as_mut().unwrap();
        
        // Cull stagnant species
        let orphaned = cull_stagnant_species(species_list, params);
        
        // Reassign orphaned individuals to nearest compatible species
        let mut next_id = get_next_species_id();
        for orphan_idx in orphaned {
            if let Some(indiv) = peeps.get(orphan_idx) {
                let species_id = assign_to_species(
                    &indiv.genome,
                    species_list,
                    params,
                    &mut next_id,
                );
                if let Some(species) = species_list.iter_mut().find(|s| s.id == species_id) {
                    species.add_member(orphan_idx);
                }
            }
        }
        
        // Limit species sizes
        let fitness_map: HashMap<u16, f32> = parents.iter()
            .map(|(idx, fitness)| (*idx, *fitness))
            .collect();
        for species in species_list.iter_mut() {
            limit_species_size(species, &fitness_map, params);
        }
        
        // Update species representatives periodically
        update_species_representatives(species_list, peeps, generation, 5);
    }

    // Assemble parent genomes
    parent_genomes.reserve(parents.len());
    for (index, _score) in &parents {
        if let Some(indiv) = peeps.get(*index) {
            parent_genomes.push(indiv.genome.clone());
        }
    }

    println!("Gen {}, {} survivors", generation, parent_genomes.len());
    crate::analysis::append_epoch_log(generation, parent_genomes.len() as u32, murder_count, peeps, params);

    // Reset metabolic tracking for all individuals at start of new generation (Phase 2)
    for index in 1..=params.population {
        if let Some(indiv) = peeps.get_mut(index as u16) {
            if indiv.alive {
                // Reward survivors with energy
                reward_energy(indiv, params);
            }
            // Reset tracking for new generation
            reset_metabolic_tracking(indiv, params);
            reset_homeostasis(indiv, params);
        }
    }
    
    if !parent_genomes.is_empty() {
        initialize_new_generation(&parent_genomes, generation + 1, grid, signals, peeps, params);
    } else {
        initialize_generation_0(grid, signals, peeps, params);
    }

    parent_genomes.len() as u32
}

