// speciation.rs
// Phase 6: NEAT-style speciation for protecting variable-length genome innovations
// Based on research document Section 6.2 (sources 33, 35)

use crate::genome_neurons::{Gene, Genome};
use crate::params::Params;
use std::sync::Mutex;
use std::collections::HashMap;

// Global species list (protected by Mutex for thread safety)
// This persists across generations
static SPECIES_LIST: Mutex<Option<Vec<Species>>> = Mutex::new(None);
static NEXT_SPECIES_ID: Mutex<u32> = Mutex::new(1);

/// Check if two genes match in topology (source and sink, ignoring weight)
/// This is used for NEAT compatibility distance calculation
#[inline]
fn genes_match_topology(g1: &Gene, g2: &Gene) -> bool {
    g1.source_type == g2.source_type
        && g1.source_num == g2.source_num
        && g1.sink_type == g2.sink_type
        && g1.sink_num == g2.sink_num
}

/// Calculate genetic compatibility distance between two genomes
/// Based on NEAT algorithm (research document Section 6.2, source 33)
/// 
/// Formula: δ = (c1*E/N) + (c2*D/N) + (c3*W)
/// where:
/// - E = excess gene count (genes beyond matching region in longer genome)
/// - D = disjoint gene count (genes that don't match between genomes)
/// - N = length of longer genome (normalization factor)
/// - W = average weight difference for matching genes
/// - c1, c2, c3 = configurable coefficients
/// 
/// # Arguments
/// * `genome1` - First genome
/// * `genome2` - Second genome
/// * `params` - Simulation parameters (contains speciation coefficients)
/// 
/// # Returns
/// Compatibility distance (lower = more similar, 0.0 = identical topology)
pub fn genetic_compatibility_distance(
    genome1: &Genome,
    genome2: &Genome,
    params: &Params,
) -> f32 {
    // Safety: handle empty genomes
    if genome1.is_empty() && genome2.is_empty() {
        return 0.0;
    }
    if genome1.is_empty() || genome2.is_empty() {
        // One genome is empty - all genes are excess/disjoint
        let n = genome1.len().max(genome2.len()) as f32;
        if n == 0.0 {
            return 0.0;
        }
        return params.excess_coefficient * n / n; // All genes are excess
    }
    
    // Find matching genes by topology (source/sink, ignoring weight)
    // Use a simple matching algorithm: for each gene in genome1, find first match in genome2
    let mut matched_indices_g2 = vec![false; genome2.len()];
    let mut matching_pairs: Vec<(usize, usize)> = Vec::new(); // (g1_idx, g2_idx)
    
    for (i, gene1) in genome1.iter().enumerate() {
        for (j, gene2) in genome2.iter().enumerate() {
            if !matched_indices_g2[j] && genes_match_topology(gene1, gene2) {
                matching_pairs.push((i, j));
                matched_indices_g2[j] = true;
                break; // Match each gene only once
            }
        }
    }
    
    // Calculate components
    let n = genome1.len().max(genome2.len()) as f32;
    if n == 0.0 {
        return 0.0;
    }
    
    // Excess genes: genes in longer genome beyond the matching region
    let num_matches = matching_pairs.len();
    let excess_count = if genome1.len() > genome2.len() {
        (genome1.len() - num_matches) as f32
    } else if genome2.len() > genome1.len() {
        (genome2.len() - num_matches) as f32
    } else {
        0.0
    };
    
    // Disjoint genes: genes that don't match (total - matches - excess)
    // Actually, disjoint = genes in shorter genome that don't match
    let shorter_len = genome1.len().min(genome2.len());
    let disjoint_count = (shorter_len - num_matches) as f32;
    
    // Average weight difference for matching genes
    let weight_diff_sum: f32 = matching_pairs.iter()
        .map(|(i, j)| {
            let w1 = genome1[*i].weight_as_float();
            let w2 = genome2[*j].weight_as_float();
            (w1 - w2).abs()
        })
        .sum();
    
    let avg_weight_diff = if matching_pairs.is_empty() {
        0.0
    } else {
        weight_diff_sum / matching_pairs.len() as f32
    };
    
    // Calculate compatibility distance
    let excess_term = params.excess_coefficient * excess_count / n;
    let disjoint_term = params.disjoint_coefficient * disjoint_count / n;
    let weight_term = params.weight_coefficient * avg_weight_diff;
    
    excess_term + disjoint_term + weight_term
}

/// Species structure for tracking groups of compatible genomes
/// Based on research document Section 6.2, source 35
#[derive(Debug, Clone)]
pub struct Species {
    pub id: u32,                        // Unique species identifier
    pub representative_genome: Genome,  // Representative genome for comparison
    pub members: Vec<u16>,              // Indices of individuals in this species
    pub age: u32,                       // Generations since species creation
    pub best_fitness: f32,              // Best fitness achieved by any member
    pub stagnation: u32,                // Generations without improvement
    pub last_improved_generation: u32,  // Generation when best_fitness was last improved
}

impl Species {
    /// Create a new species with a representative genome
    pub fn new(id: u32, representative_genome: Genome, generation: u32) -> Self {
        Species {
            id,
            representative_genome,
            members: Vec::new(),
            age: 0,
            best_fitness: 0.0,
            stagnation: 0,
            last_improved_generation: generation,
        }
    }
    
    /// Check if a genome is compatible with this species
    pub fn is_compatible(&self, genome: &Genome, params: &Params) -> bool {
        let distance = genetic_compatibility_distance(
            genome,
            &self.representative_genome,
            params,
        );
        distance <= params.compatibility_threshold
    }
    
    /// Add a member to this species
    pub fn add_member(&mut self, individual_index: u16) {
        if !self.members.contains(&individual_index) {
            self.members.push(individual_index);
        }
    }
    
    /// Remove a member from this species
    pub fn remove_member(&mut self, individual_index: u16) {
        self.members.retain(|&idx| idx != individual_index);
    }
    
    /// Clear all members (used when resetting for new generation)
    pub fn clear_members(&mut self) {
        self.members.clear();
    }
    
    /// Update species statistics
    pub fn update_stats(&mut self, generation: u32, current_best_fitness: f32) {
        self.age = generation - (self.last_improved_generation - self.age);
        
        if current_best_fitness > self.best_fitness {
            self.best_fitness = current_best_fitness;
            self.stagnation = 0;
            self.last_improved_generation = generation;
        } else {
            self.stagnation += 1;
        }
    }
}

/// Assign a genome to a species, creating a new species if no compatible one exists
/// Based on research document Section 6.2, source 35
/// 
/// # Arguments
/// * `genome` - Genome to assign
/// * `species_list` - List of existing species
/// * `params` - Simulation parameters
/// * `next_species_id` - Next available species ID (will be incremented if new species created)
/// 
/// # Returns
/// Species ID that the genome was assigned to
pub fn assign_to_species(
    genome: &Genome,
    species_list: &mut Vec<Species>,
    params: &Params,
    next_species_id: &mut u32,
) -> u32 {
    // Try to find a compatible existing species
    for species in species_list.iter() {
        if species.is_compatible(genome, params) {
            return species.id;
        }
    }
    
    // No compatible species found - create new species
    // Defensive check: prevent excessive species creation
    if species_list.len() > 10000 {
        eprintln!("ERROR: Too many species created ({}). Stopping to prevent crash.", species_list.len());
        eprintln!("  This may indicate a bug in compatibility distance calculation or threshold.");
        eprintln!("  Compatibility threshold: {}", params.compatibility_threshold);
        panic!("Species list overflow: {} species created. Check compatibility threshold and distance calculation.", 
            species_list.len());
    }
    
    let new_id = *next_species_id;
    *next_species_id += 1;
    let new_species = Species::new(new_id, genome.clone(), 0); // Generation will be set later
    species_list.push(new_species);
    new_id
}

/// Calculate shared fitness for an individual within a species
/// Based on research document Section 6.2, source 33 (NEAT fitness sharing)
/// 
/// Fitness sharing prevents large species from dominating selection by dividing
/// individual fitness by species size. This encourages diversity across species.
/// 
/// # Arguments
/// * `raw_fitness` - Individual's raw fitness score
/// * `species_size` - Number of members in the species
/// 
/// # Returns
/// Shared fitness value
#[inline]
pub fn shared_fitness(raw_fitness: f32, species_size: usize) -> f32 {
    if species_size == 0 {
        return 0.0;
    }
    raw_fitness / species_size as f32
}

/// Adjust fitness based on species age and stagnation
/// Optional enhancement: boost young species, penalize old stagnant species
/// 
/// # Arguments
/// * `shared_fitness` - Fitness after sharing
/// * `species_age` - Generations since species creation
/// * `species_stagnation` - Generations without improvement
/// * `params` - Simulation parameters
/// 
/// # Returns
/// Adjusted fitness value
pub fn adjust_fitness_for_age(
    shared_fitness: f32,
    species_age: u32,
    species_stagnation: u32,
    params: &Params,
) -> f32 {
    let mut adjusted = shared_fitness;
    
    // Boost young species (first few generations) to encourage exploration
    if species_age < 10 {
        adjusted *= 1.1; // 10% boost for young species
    }
    
    // Penalize stagnant species
    if species_stagnation > params.species_stagnation_threshold / 2 {
        adjusted *= 0.9; // 10% penalty for stagnation
    }
    
    adjusted
}

/// Update species statistics (age, best fitness, stagnation)
/// 
/// # Arguments
/// * `species` - Species to update
/// * `generation` - Current generation number
/// * `current_best_fitness` - Best fitness in species this generation
pub fn update_species_stats(
    species: &mut Species,
    generation: u32,
    current_best_fitness: f32,
) {
    species.age = generation.saturating_sub(species.last_improved_generation.saturating_sub(species.age));
    
    if current_best_fitness > species.best_fitness {
        species.best_fitness = current_best_fitness;
        species.stagnation = 0;
        species.last_improved_generation = generation;
    } else {
        species.stagnation += 1;
    }
}

/// Cull stagnant species that haven't improved for threshold generations
/// Based on research document Section 6.2, source 35
/// 
/// # Arguments
/// * `species_list` - List of species to evaluate
/// * `params` - Simulation parameters
/// 
/// # Returns
/// Vector of individual indices from extinct species (to be reassigned)
pub fn cull_stagnant_species(
    species_list: &mut Vec<Species>,
    params: &Params,
) -> Vec<u16> {
    let mut orphaned_individuals = Vec::new();
    let mut species_to_remove = Vec::new();
    
    for (idx, species) in species_list.iter().enumerate() {
        // Don't remove if it's the only species (prevent total extinction)
        if species_list.len() <= 1 {
            break;
        }
        
        // Remove if stagnant and below minimum size
        if species.stagnation >= params.species_stagnation_threshold
            && species.members.len() < params.min_species_size
        {
            orphaned_individuals.extend_from_slice(&species.members);
            species_to_remove.push(idx);
        }
    }
    
    // Remove extinct species (in reverse order to maintain indices)
    for &idx in species_to_remove.iter().rev() {
        species_list.remove(idx);
    }
    
    orphaned_individuals
}

/// Limit species size by keeping only top performers
/// 
/// # Arguments
/// * `species` - Species to limit
/// * `fitness_scores` - Map from individual index to fitness score
/// * `params` - Simulation parameters
pub fn limit_species_size(
    species: &mut Species,
    fitness_scores: &std::collections::HashMap<u16, f32>,
    params: &Params,
) {
    if species.members.len() <= params.max_species_size {
        return; // No culling needed
    }
    
    // Sort members by fitness (descending)
    let mut members_with_fitness: Vec<(u16, f32)> = species.members.iter()
        .map(|&idx| {
            let fitness = fitness_scores.get(&idx).copied().unwrap_or(0.0);
            (idx, fitness)
        })
        .collect();
    
    members_with_fitness.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Keep top max_species_size members
    species.members = members_with_fitness.into_iter()
        .take(params.max_species_size)
        .map(|(idx, _)| idx)
        .collect();
}

/// Update species representatives periodically
/// Representatives are used for compatibility comparison
/// 
/// # Arguments
/// * `species_list` - List of species to update
/// * `peeps` - Population of individuals
/// * `generation` - Current generation
/// * `update_interval` - Update every N generations (default: 5)
pub fn update_species_representatives(
    species_list: &mut Vec<Species>,
    peeps: &crate::peeps::Peeps,
    generation: u32,
    update_interval: u32,
) {
    // Update representatives every N generations
    if generation % update_interval != 0 {
        return;
    }
    
    for species in species_list.iter_mut() {
        if species.members.is_empty() {
            continue;
        }
        
        // Use the best-performing member as new representative
        // For now, use first member (can be enhanced to use best fitness)
        if let Some(&representative_idx) = species.members.first() {
            if let Some(indiv) = peeps.get(representative_idx) {
                species.representative_genome = indiv.genome.clone();
            }
        }
    }
}

/// Get or initialize the global species list
pub fn get_species_list() -> std::sync::MutexGuard<'static, Option<Vec<Species>>> {
    let mut list = SPECIES_LIST.lock().unwrap();
    if list.is_none() {
        *list = Some(Vec::new());
    }
    list
}

/// Reset species list (called at generation 0)
pub fn reset_species_list() {
    let mut list = SPECIES_LIST.lock().unwrap();
    *list = Some(Vec::new());
    let mut next_id = NEXT_SPECIES_ID.lock().unwrap();
    *next_id = 1;
}

/// Get next species ID (increments automatically)
pub fn get_next_species_id() -> u32 {
    let mut next_id = NEXT_SPECIES_ID.lock().unwrap();
    
    // Defensive check: prevent integer overflow
    if *next_id >= u32::MAX - 1000 {
        eprintln!("WARNING: Species ID approaching overflow ({}). Resetting to 1.", *next_id);
        *next_id = 1;
    }
    
    let id = *next_id;
    *next_id += 1;
    id
}

/// Species statistics for epoch logging
#[derive(Debug, Clone, Default)]
pub struct SpeciesStatistics {
    pub num_species: u32,
    pub avg_species_size: f32,
    pub largest_species: u32,
    pub smallest_species: u32,
    pub new_species: u32,
    pub extinct_species: u32,
    pub avg_species_age: f32,
    pub avg_stagnation: f32,
}

/// Calculate species statistics for epoch logging
/// 
/// # Arguments
/// * `_generation` - Current generation number (used for future enhancements)
/// * `previous_species_count` - Number of species in previous generation (for calculating new/extinct)
/// 
/// # Returns
/// SpeciesStatistics struct with all calculated metrics
pub fn calculate_species_statistics(
    _generation: u32,
    previous_species_count: u32,
) -> SpeciesStatistics {
    let list_guard = get_species_list();
    let list = match list_guard.as_ref() {
        Some(l) => l,
        None => return SpeciesStatistics::default(),
    };
    
    if list.is_empty() {
        return SpeciesStatistics::default();
    }
    
    let num_species = list.len() as u32;
    
    // Defensive check: prevent corrupted statistics from being logged
    if num_species > 10000 {
        eprintln!("ERROR: Suspiciously high species count: {} (generation {}). Species list may be corrupted!", 
            num_species, _generation);
        eprintln!("  Species list length: {}", list.len());
        eprintln!("  Returning default statistics to prevent crash.");
        return SpeciesStatistics::default();
    }
    
    // Calculate species sizes
    let mut sizes: Vec<usize> = list.iter()
        .map(|s| s.members.len())
        .collect();
    
    if sizes.is_empty() {
        return SpeciesStatistics::default();
    }
    
    let total_members: usize = sizes.iter().sum();
    let avg_species_size = total_members as f32 / num_species as f32;
    
    sizes.sort();
    let largest_species = *sizes.last().unwrap_or(&0) as u32;
    let smallest_species = *sizes.first().unwrap_or(&0) as u32;
    
    // Calculate new and extinct species
    // New species: species created this generation (age == 0)
    // Note: Age is incremented in update_species_stats, so age == 0 means just created
    let new_species = list.iter()
        .filter(|s| s.age == 0)
        .count() as u32;
    
    // Extinct species: species that existed in previous generation but not now
    // This is approximate - we compare current count to previous count
    // If previous count was higher, the difference is extinct species
    let extinct_species = if previous_species_count > num_species {
        previous_species_count - num_species
    } else {
        0
    };
    
    // Calculate average age and stagnation
    let total_age: u32 = list.iter().map(|s| s.age).sum();
    let avg_species_age = total_age as f32 / num_species as f32;
    
    let total_stagnation: u32 = list.iter().map(|s| s.stagnation).sum();
    let avg_stagnation = total_stagnation as f32 / num_species as f32;
    
    SpeciesStatistics {
        num_species,
        avg_species_size,
        largest_species,
        smallest_species,
        new_species,
        extinct_species,
        avg_species_age,
        avg_stagnation,
    }
}

/// Assign all survivors to species and return species assignments
/// 
/// # Arguments
/// * `survivors` - Vector of (individual_index, fitness_score) tuples
/// * `peeps` - Population of individuals
/// * `params` - Simulation parameters
/// 
/// # Returns
/// Map from individual index to species ID
pub fn assign_survivors_to_species(
    survivors: &[(u16, f32)],
    peeps: &crate::peeps::Peeps,
    params: &Params,
) -> HashMap<u16, u32> {
    let mut assignments = HashMap::new();
    let mut list_guard = get_species_list();
    let list = list_guard.as_mut().unwrap();
    let mut next_id = get_next_species_id();
    
    // Clear all species members for new generation
    for species in list.iter_mut() {
        species.clear_members();
    }
    
    // Performance optimization: For small populations, use direct assignment
    // For large populations, we could parallelize this, but for now keep it simple
    // Assign each survivor to a species
    for &(idx, _fitness) in survivors {
        if let Some(indiv) = peeps.get(idx) {
            let species_id = assign_to_species(&indiv.genome, list, params, &mut next_id);
            assignments.insert(idx, species_id);
            
            // Add member to species (use HashMap lookup for O(1) instead of O(n) find)
            // Build index map for faster lookup
            for species in list.iter_mut() {
                if species.id == species_id {
                    species.add_member(idx);
                    break;
                }
            }
        }
    }
    
    assignments
}

/// Select parents using speciation (proportional selection from each species)
/// Based on research document Section 6.2, source 33
/// 
/// # Arguments
/// * `survivors` - Vector of (individual_index, raw_fitness) tuples
/// * `species_assignments` - Map from individual index to species ID
/// * `species_list` - List of species
/// * `num_parents_needed` - Number of parents to select
/// * `params` - Simulation parameters
/// 
/// # Returns
/// Vector of selected parent indices
pub fn select_parents_with_speciation(
    survivors: &[(u16, f32)],
    species_assignments: &HashMap<u16, u32>,
    species_list: &[Species],
    num_parents_needed: usize,
    params: &Params,
) -> Vec<u16> {
    if species_list.is_empty() {
        // Fallback: no species, select top performers
        let mut sorted = survivors.to_vec();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        return sorted.into_iter()
            .take(num_parents_needed)
            .map(|(idx, _)| idx)
            .collect();
    }
    
    // Calculate shared fitness for each individual
    let mut shared_fitness_map: HashMap<u16, f32> = HashMap::new();
    let mut species_fitness_sum: HashMap<u32, f32> = HashMap::new();
    
    for &(idx, raw_fitness) in survivors {
        if let Some(&species_id) = species_assignments.get(&idx) {
            if let Some(species) = species_list.iter().find(|s| s.id == species_id) {
                let shared = shared_fitness(raw_fitness, species.members.len());
                let adjusted = adjust_fitness_for_age(
                    shared,
                    species.age,
                    species.stagnation,
                    params,
                );
                shared_fitness_map.insert(idx, adjusted);
                *species_fitness_sum.entry(species_id).or_insert(0.0) += adjusted;
            }
        }
    }
    
    // Calculate total fitness across all species
    let total_fitness: f32 = species_fitness_sum.values().sum();
    if total_fitness == 0.0 {
        // Fallback: equal probability
        let mut selected = Vec::new();
        for _ in 0..num_parents_needed {
            if let Some(&(idx, _)) = survivors.get(crate::random::random_uint_range(0, survivors.len() as u32) as usize) {
                selected.push(idx);
            }
        }
        return selected;
    }
    
    // Select parents proportionally from each species
    let mut selected_parents = Vec::new();
    let mut remaining = num_parents_needed;
    
    // Ensure each species gets at least one parent if it has members
    let mut species_parents: HashMap<u32, Vec<u16>> = HashMap::new();
    
    for species in species_list {
        if species.members.is_empty() {
            continue;
        }
        
        let species_fitness = species_fitness_sum.get(&species.id).copied().unwrap_or(0.0);
        let species_share = species_fitness / total_fitness;
        let num_from_species = (species_share * num_parents_needed as f32).max(1.0) as usize;
        let num_from_species = num_from_species.min(species.members.len()).min(remaining);
        
        // Get members of this species with their shared fitness
        let mut species_members: Vec<(u16, f32)> = species.members.iter()
            .filter_map(|&idx| {
                shared_fitness_map.get(&idx).map(|&fitness| (idx, fitness))
            })
            .collect();
        
        // Sort by fitness (descending)
        species_members.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Select top performers from this species
        for (idx, _) in species_members.into_iter().take(num_from_species) {
            species_parents.entry(species.id).or_insert_with(Vec::new).push(idx);
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
        
        if remaining == 0 {
            break;
        }
    }
    
    // Flatten species_parents into selected_parents
    for parents in species_parents.values() {
        selected_parents.extend_from_slice(parents);
    }
    
    // If we still need more parents, select randomly from remaining
    while selected_parents.len() < num_parents_needed && !survivors.is_empty() {
        let idx = crate::random::random_uint_range(0, survivors.len() as u32) as usize;
        if let Some(&(parent_idx, _)) = survivors.get(idx) {
            if !selected_parents.contains(&parent_idx) {
                selected_parents.push(parent_idx);
            }
        }
    }
    
    selected_parents.truncate(num_parents_needed);
    selected_parents
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome_neurons::NEURON;

    fn make_test_params() -> Params {
        let mut params = Params::default();
        params.excess_coefficient = 1.0;
        params.disjoint_coefficient = 1.0;
        params.weight_coefficient = 0.4;
        params
    }

    #[test]
    fn test_compatibility_distance_identical() {
        let params = make_test_params();
        let gene = Gene {
            source_type: NEURON,
            source_num: 0,
            sink_type: NEURON,
            sink_num: 1,
            weight: 8192, // 1.0
        };
        let genome1 = vec![gene];
        let genome2 = vec![gene];
        
        let distance = genetic_compatibility_distance(&genome1, &genome2, &params);
        // Identical genomes should have distance = 0 (only weight difference, which is 0)
        assert!(distance < 0.0001);
    }

    #[test]
    fn test_compatibility_distance_excess_genes() {
        let params = make_test_params();
        let gene1 = Gene {
            source_type: NEURON,
            source_num: 0,
            sink_type: NEURON,
            sink_num: 1,
            weight: 8192,
        };
        let gene2 = Gene {
            source_type: NEURON,
            source_num: 1,
            sink_type: NEURON,
            sink_num: 2,
            weight: 8192,
        };
        let genome1 = vec![gene1];
        let genome2 = vec![gene1, gene2]; // genome2 has one excess gene
        
        let distance = genetic_compatibility_distance(&genome1, &genome2, &params);
        // Should have excess term: 1.0 * 1 / 2 = 0.5
        assert!(distance > 0.4 && distance < 0.6);
    }

    #[test]
    fn test_compatibility_distance_disjoint_genes() {
        let params = make_test_params();
        let gene1 = Gene {
            source_type: NEURON,
            source_num: 0,
            sink_type: NEURON,
            sink_num: 1,
            weight: 8192,
        };
        let gene2 = Gene {
            source_type: NEURON,
            source_num: 1,
            sink_type: NEURON,
            sink_num: 2,
            weight: 8192,
        };
        let genome1 = vec![gene1];
        let genome2 = vec![gene2]; // Different genes, no matches
        
        let distance = genetic_compatibility_distance(&genome1, &genome2, &params);
        // Should have disjoint term: 1.0 * 1 / 1 = 1.0
        assert!(distance > 0.9 && distance < 1.1);
    }

    #[test]
    fn test_compatibility_distance_weight_difference() {
        let params = make_test_params();
        let gene1 = Gene {
            source_type: NEURON,
            source_num: 0,
            sink_type: NEURON,
            sink_num: 1,
            weight: 8192, // 1.0
        };
        let gene2 = Gene {
            source_type: NEURON,
            source_num: 0,
            sink_type: NEURON,
            sink_num: 1,
            weight: 16384, // 2.0 (different weight, same topology)
        };
        let genome1 = vec![gene1];
        let genome2 = vec![gene2];
        
        let distance = genetic_compatibility_distance(&genome1, &genome2, &params);
        // Should have weight term: 0.4 * |1.0 - 2.0| = 0.4
        assert!(distance > 0.35 && distance < 0.45);
    }
}

