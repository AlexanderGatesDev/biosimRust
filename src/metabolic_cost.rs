// metabolic_cost.rs
// Metabolic cost calculation for energy-based fitness (Phase 2)

use crate::indiv::Indiv;
use crate::params::Params;

/// Calculate the metabolic cost for a single simulation step
/// - Structural cost: maintenance of synapses (per step)
/// - Signaling cost: spikes in this step only
/// Note: This is called each step, and costs accumulate in metabolic_cost_accumulated
pub fn calculate_metabolic_cost(indiv: &Indiv, params: &Params) -> f32 {
    // Static cost: maintenance of synapses per step
    // Each synapse requires energy to maintain its structure each step
    let structural_cost = indiv.nnet.connections.len() as f32 
        * params.metabolic_cost_per_synapse;
    
    // Dynamic cost: signaling (spikes in this step only)
    // Use spikes_this_step to avoid double-counting
    let signaling_cost = indiv.spikes_this_step as f32 
        * params.metabolic_cost_per_spike;
    
    structural_cost + signaling_cost
}

/// Update metabolic energy for an individual
/// Returns true if the individual should die from energy depletion
/// Note: Does NOT set alive=false here - that's handled by drain_death_queue()
/// Resets spikes_this_step after calculating cost
pub fn update_metabolic_energy(indiv: &mut Indiv, params: &Params) -> bool {
    let cost = calculate_metabolic_cost(indiv, params);
    indiv.metabolic_cost_accumulated += cost;
    indiv.metabolic_energy -= cost;
    
    // Reset spikes_this_step for next step
    indiv.spikes_this_step = 0;
    
    // Return true if energy depleted (death will be handled by queue_for_death)
    indiv.metabolic_energy <= 0.0
}

/// Reset metabolic tracking for a new generation
pub fn reset_metabolic_tracking(indiv: &mut Indiv, params: &Params) {
    // Reset accumulated cost and spike counts
    indiv.metabolic_cost_accumulated = 0.0;
    indiv.spike_count = 0;
    indiv.spikes_this_step = 0;
    
    // Restore energy to steps_per_generation
    // This ensures individuals have enough energy to survive the full generation
    // regardless of how many steps the user configures
    indiv.metabolic_energy = params.steps_per_generation as f32;
}

/// Add energy reward for successful survival
pub fn reward_energy(indiv: &mut Indiv, params: &Params) {
    indiv.metabolic_energy += params.energy_reward_per_success;
    // Cap energy at a reasonable maximum (e.g., 2x initial)
    let max_energy = params.initial_energy * 2.0;
    if indiv.metabolic_energy > max_energy {
        indiv.metabolic_energy = max_energy;
    }
}

