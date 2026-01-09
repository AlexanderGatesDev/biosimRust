// homeostasis.rs
// Homeostatic plasticity mechanisms to stabilize neural networks after duplication events

use crate::indiv::Indiv;
use crate::params::Params;

/// Update firing rate using exponential moving average (EMA)
/// Much more efficient than maintaining a full history window
/// EMA: new_rate = alpha * current_value + (1 - alpha) * old_rate
/// where alpha = 1.0 / window_size for equivalent averaging
pub fn update_firing_rate(neuron: &mut crate::genome_neurons::Neuron, fired: bool, params: &Params) {
    let firing_value = if fired { 1.0 } else { 0.0 };
    
    // Calculate EMA alpha based on window size
    // For a window of N, we want alpha = 1/N to get equivalent averaging
    // But we use a slightly larger alpha for faster initial convergence
    let alpha = 1.0 / (params.firing_rate_window as f32).max(1.0);
    
    // Exponential moving average: new = alpha * current + (1 - alpha) * old
    // This is mathematically equivalent to a moving average but O(1) instead of O(n)
    neuron.current_firing_rate = alpha * firing_value + (1.0 - alpha) * neuron.current_firing_rate;
}

/// Update homeostatic scaling for a neuron based on firing rate
/// If firing rate is below target, increase scaling (strengthen connections)
/// If firing rate is above target, decrease scaling (weaken connections)
/// Returns the new homeostatic_scale value
pub fn update_homeostatic_scale(neuron: &mut crate::genome_neurons::Neuron, params: &Params) -> f32 {
    // With EMA, we always have a valid firing rate estimate
    // No need to wait for history to fill up
    
    let error = neuron.current_firing_rate - neuron.target_firing_rate;
    
    // Update scaling: scale increases if firing rate is too low, decreases if too high
    // Use exponential update: scale *= (1 + alpha * error)
    // Clamp to reasonable range [0.1, 10.0] to prevent extreme values
    let scale_change = 1.0 + params.homeostasis_alpha * error;
    neuron.homeostatic_scale *= scale_change;
    
    // Clamp to prevent extreme values
    neuron.homeostatic_scale = neuron.homeostatic_scale.max(0.1).min(10.0);
    
    neuron.homeostatic_scale
}

/// Apply homeostatic plasticity to an individual's neural network
/// Updates firing rates and adjusts connection weights based on homeostatic scaling
pub fn update_homeostasis(indiv: &mut Indiv, params: &Params) {
    if !params.homeostasis_enabled {
        return;
    }
    
    // First, update firing rates for all neurons based on their outputs
    // A neuron "fired" if its output exceeds a threshold (e.g., 0.1)
    for neuron in &mut indiv.nnet.neurons {
        if neuron.driven {
            let fired = neuron.output.abs() > 0.1;
            update_firing_rate(neuron, fired, params);
        }
    }
    
    // Then, update homeostatic scaling for each neuron
    for neuron in &mut indiv.nnet.neurons {
        if neuron.driven {
            update_homeostatic_scale(neuron, params);
        }
    }
    
    // Note: Homeostatic scaling is applied during feed-forward computation
    // We don't modify weights directly here to avoid compounding effects
    // The homeostatic_scale values are used in feed_forward.rs when computing neuron inputs
}

/// Reset homeostatic state for a new generation
/// Clears firing rate history and resets scaling to 1.0
pub fn reset_homeostasis(indiv: &mut Indiv, params: &Params) {
    if !params.homeostasis_enabled {
        return;
    }
    
    for neuron in &mut indiv.nnet.neurons {
        neuron.current_firing_rate = 0.0;
        neuron.homeostatic_scale = 1.0;
    }
}

