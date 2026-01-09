// multi_objective.rs
// Multi-objective optimization using NSGA-II (Non-dominated Sorting Genetic Algorithm II)
// Phase 5: Maintain diversity while selecting for performance

use crate::params::Params;
use crate::peeps::Peeps;

/// Represents multiple fitness objectives for an individual
#[derive(Debug, Clone)]
pub struct FitnessObjectives {
    pub survival_score: f32,      // Primary: survival performance (0.0-1.0)
    pub efficiency: f32,           // Secondary: metabolic efficiency (higher = better, normalized)
    pub diversity_contribution: f32, // Tertiary: genetic diversity contribution (higher = more unique)
}

impl FitnessObjectives {
    /// Create fitness objectives from an individual
    pub fn from_indiv(
        index: u16,
        survival_score: f32,
        metabolic_cost_accumulated: f32,
        steps_per_generation: u32,
        peeps: &Peeps,
        params: &Params,
    ) -> Self {
        // Calculate efficiency: inverse of normalized metabolic cost
        // Lower cost = higher efficiency
        let steps = steps_per_generation as f32;
        let normalized_cost = (metabolic_cost_accumulated / steps).min(1.0);
        let efficiency = 1.0 - normalized_cost; // Invert so higher is better
        
        // Calculate diversity contribution: average dissimilarity to other survivors
        // This is expensive, so we'll use a simplified version
        let diversity_contribution = Self::calculate_diversity_contribution(
            index,
            peeps,
            params,
        );
        
        FitnessObjectives {
            survival_score,
            efficiency,
            diversity_contribution,
        }
    }
    
    /// Calculate diversity contribution using a fast approximation
    /// Uses genome length difference as a proxy for diversity (much faster than similarity)
    /// Returns normalized diversity score (0.0-1.0)
    fn calculate_diversity_contribution(
        index: u16,
        peeps: &Peeps,
        params: &Params,
    ) -> f32 {
        // Fast approximation: use genome length as diversity proxy
        // Longer/shorter genomes are considered more diverse
        // This avoids expensive genome similarity calculations
        
        if let Some(indiv) = peeps.get(index) {
            let genome_len = indiv.genome.len() as f32;
            let max_len = params.genome_max_length as f32;
            
            // Normalize length to 0.0-1.0 range
            let normalized_len = (genome_len / max_len).min(1.0);
            
            // Sample a few other individuals to see how different our length is
            const SAMPLE_SIZE: usize = 3; // Reduced from 10 for performance
            let mut length_diffs = Vec::with_capacity(SAMPLE_SIZE);
            
            for _ in 0..SAMPLE_SIZE {
                let other_index = crate::random::random_uint_range(1, params.population) as u16;
                if other_index != index {
                    if let Some(other) = peeps.get(other_index) {
                        let other_len = other.genome.len() as f32;
                        let other_normalized = (other_len / max_len).min(1.0);
                        // Difference in normalized length = diversity proxy
                        length_diffs.push((normalized_len - other_normalized).abs());
                    }
                }
            }
            
            if length_diffs.is_empty() {
                // If no comparisons, use position in length distribution as diversity
                // Middle lengths are more common, extremes are more diverse
                return if normalized_len < 0.3 || normalized_len > 0.7 {
                    0.8 // Extreme lengths = high diversity
                } else {
                    0.5 // Middle lengths = moderate diversity
                };
            }
            
            // Average length difference = diversity contribution
            let avg_diff = length_diffs.iter().sum::<f32>() / length_diffs.len() as f32;
            // Scale to 0.0-1.0 range (max difference is 1.0)
            avg_diff.min(1.0)
        } else {
            0.5 // Default moderate diversity
        }
    }
    
    /// Check if this solution dominates another (Pareto dominance)
    /// Solution A dominates solution B if A is better in at least one objective
    /// and not worse in any objective
    pub fn dominates(&self, other: &FitnessObjectives) -> bool {
        // For all objectives, we want to maximize (higher is better)
        let better_survival = self.survival_score >= other.survival_score;
        let better_efficiency = self.efficiency >= other.efficiency;
        let better_diversity = self.diversity_contribution >= other.diversity_contribution;
        
        // At least one must be strictly better
        let at_least_one_better = 
            self.survival_score > other.survival_score ||
            self.efficiency > other.efficiency ||
            self.diversity_contribution > other.diversity_contribution;
        
        // All must be at least as good
        let all_at_least_as_good = better_survival && better_efficiency && better_diversity;
        
        at_least_one_better && all_at_least_as_good
    }
}

/// Represents an individual with its fitness objectives and metadata
#[derive(Debug, Clone)]
pub struct IndividualWithObjectives {
    pub index: u16,
    pub objectives: FitnessObjectives,
    pub rank: usize,              // Non-dominated sorting rank (lower = better)
    pub crowding_distance: f32,  // Crowding distance for diversity preservation
}

/// Perform non-dominated sorting (NSGA-II front assignment)
/// Returns individuals grouped by Pareto front (rank)
pub fn non_dominated_sort(
    individuals: &mut [IndividualWithObjectives],
) -> Vec<Vec<usize>> {
    let n = individuals.len();
    if n == 0 {
        return Vec::new();
    }
    
    // Initialize all ranks to a high value (unassigned)
    for indiv in individuals.iter_mut() {
        indiv.rank = usize::MAX;
    }
    
    let mut fronts: Vec<Vec<usize>> = Vec::new();
    let mut dominated_by: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut domination_count: Vec<usize> = vec![0; n];
    
    // Build domination relationships
    for i in 0..n {
        for j in 0..n {
            if i != j {
                if individuals[i].objectives.dominates(&individuals[j].objectives) {
                    dominated_by[i].push(j);
                } else if individuals[j].objectives.dominates(&individuals[i].objectives) {
                    domination_count[i] += 1;
                }
            }
        }
    }
    
    // Find first front (non-dominated solutions)
    let mut current_front: Vec<usize> = Vec::new();
    for i in 0..n {
        if domination_count[i] == 0 {
            current_front.push(i);
            individuals[i].rank = 0;
        }
    }
    
    if current_front.is_empty() {
        // All solutions are dominated (shouldn't happen, but handle gracefully)
        for i in 0..n {
            individuals[i].rank = 0;
        }
        return vec![(0..n).collect()];
    }
    
    fronts.push(current_front.clone());
    
    // Find subsequent fronts
    let mut front_index = 0;
    while !current_front.is_empty() {
        let mut next_front: Vec<usize> = Vec::new();
        
        for &i in &current_front {
            for &j in &dominated_by[i] {
                domination_count[j] -= 1;
                if domination_count[j] == 0 {
                    next_front.push(j);
                    individuals[j].rank = front_index + 1;
                }
            }
        }
        
        if !next_front.is_empty() {
            fronts.push(next_front.clone());
            current_front = next_front;
            front_index += 1;
        } else {
            break;
        }
    }
    
    // Assign any remaining unassigned individuals to last front
    for i in 0..n {
        if individuals[i].rank == usize::MAX {
            individuals[i].rank = front_index + 1;
            if fronts.len() <= front_index + 1 {
                fronts.push(Vec::new());
            }
            fronts[front_index + 1].push(i);
        }
    }
    
    fronts
}

/// Calculate crowding distance for individuals in a front
/// Higher crowding distance = more unique solution (prefer for diversity)
pub fn calculate_crowding_distance(
    individuals: &mut [IndividualWithObjectives],
    front_indices: &[usize],
) {
    if front_indices.len() <= 2 {
        // If 2 or fewer individuals, assign maximum distance
        for &idx in front_indices {
            individuals[idx].crowding_distance = f32::INFINITY;
        }
        return;
    }
    
    // Initialize distances
    for &idx in front_indices {
        individuals[idx].crowding_distance = 0.0;
    }
    
    // Calculate distance for each objective
    let objectives = [
        |obj: &FitnessObjectives| obj.survival_score,
        |obj: &FitnessObjectives| obj.efficiency,
        |obj: &FitnessObjectives| obj.diversity_contribution,
    ];
    
    for get_objective in &objectives {
        // Sort front by this objective
        let mut sorted_indices: Vec<usize> = front_indices.to_vec();
        sorted_indices.sort_by(|&a, &b| {
            let val_a = get_objective(&individuals[a].objectives);
            let val_b = get_objective(&individuals[b].objectives);
            val_a.partial_cmp(&val_b).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Boundary solutions get maximum distance
        if let Some(&first) = sorted_indices.first() {
            individuals[first].crowding_distance = f32::INFINITY;
        }
        if let Some(&last) = sorted_indices.last() {
            individuals[last].crowding_distance = f32::INFINITY;
        }
        
        // Calculate range for normalization
        let min_val = get_objective(&individuals[sorted_indices[0]].objectives);
        let max_val = get_objective(&individuals[sorted_indices[sorted_indices.len() - 1]].objectives);
        let range = (max_val - min_val).max(1e-10); // Avoid division by zero
        
        // Add distance contribution for interior solutions
        for i in 1..sorted_indices.len() - 1 {
            let idx = sorted_indices[i];
            let prev_val = get_objective(&individuals[sorted_indices[i - 1]].objectives);
            let next_val = get_objective(&individuals[sorted_indices[i + 1]].objectives);
            let contribution = (next_val - prev_val) / range;
            individuals[idx].crowding_distance += contribution;
        }
    }
}

/// Select parents using NSGA-II algorithm
/// Returns indices of selected parents
pub fn nsga2_selection(
    individuals: &mut [IndividualWithObjectives],
    num_parents: usize,
) -> Vec<usize> {
    if individuals.is_empty() {
        return Vec::new();
    }
    
    // Perform non-dominated sorting
    let fronts = non_dominated_sort(individuals);
    
    // Calculate crowding distance for each front
    for front_indices in &fronts {
        calculate_crowding_distance(individuals, front_indices);
    }
    
    // Select parents: prefer lower rank (better front), then higher crowding distance (more diverse)
    let mut selected: Vec<usize> = Vec::with_capacity(num_parents);
    let mut remaining = num_parents;
    
    // Add individuals from fronts in order
    for front_indices in &fronts {
        if remaining == 0 {
            break;
        }
        
        if front_indices.len() <= remaining {
            // Add entire front
            selected.extend_from_slice(front_indices);
            remaining -= front_indices.len();
        } else {
            // Select from this front based on crowding distance
            let mut front_with_distance: Vec<(usize, f32)> = front_indices
                .iter()
                .map(|&idx| (idx, individuals[idx].crowding_distance))
                .collect();
            
            // Sort by crowding distance (descending)
            front_with_distance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            
            // Select top individuals from this front
            for (idx, _) in front_with_distance.iter().take(remaining) {
                selected.push(*idx);
            }
            remaining = 0;
        }
    }
    
    selected
}

