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

    let mut parents: Vec<(u16, f32)> = Vec::new(); // (indiv index, score)
    let mut parent_genomes: Vec<Genome> = Vec::new();

    if params.challenge != CHALLENGE_ALTRUISM {
        for index in 1..=params.population {
            if let Some(indiv) = peeps.get(index as u16) {
                let (passed, score) = passed_survival_criterion(indiv, params.challenge, params, grid);
                if passed && !indiv.nnet.connections.is_empty() {
                    parents.push((index as u16, score));
                }
            }
        }
    } else {
        // Altruism challenge
        let consider_kinship = true;
        let mut sacrifice_indexes: Vec<u16> = Vec::new();

        for index in 1..=params.population {
            if let Some(indiv) = peeps.get(index as u16) {
                let (passed, score) = passed_survival_criterion(indiv, CHALLENGE_ALTRUISM, params, grid);
                if passed && !indiv.nnet.connections.is_empty() {
                    parents.push((index as u16, score));
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

    // Sort parents by score (descending)
    parents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Assemble parent genomes
    parent_genomes.reserve(parents.len());
    for (index, _score) in &parents {
        if let Some(indiv) = peeps.get(*index) {
            parent_genomes.push(indiv.genome.clone());
        }
    }

    println!("Gen {}, {} survivors", generation, parent_genomes.len());
    crate::analysis::append_epoch_log(generation, parent_genomes.len() as u32, murder_count, peeps, params);

    if !parent_genomes.is_empty() {
        initialize_new_generation(&parent_genomes, generation + 1, grid, signals, peeps, params);
    } else {
        initialize_generation_0(grid, signals, peeps, params);
    }

    parent_genomes.len() as u32
}

