// dendritic.rs
// Phase 4.3: Dendritic integration computation
// Implements biological realism through non-linear dendritic computation

use crate::genome_neurons::Connection;
use crate::params::Params;

/// Compute dendritic output using non-linear integration
/// 
/// For biological realism:
/// - Quadratic integration (exponent=2.0): Cooperative effect, stronger response to multiple inputs
/// - Sublinear integration (exponent=0.5): Saturating effect, diminishing returns
/// - Linear integration (exponent=1.0): Standard summation
/// 
/// # Arguments
/// * `dendrite_sum` - Sum of weighted inputs to the dendrite
/// * `exponent` - Integration exponent (1.0=linear, 2.0=quadratic, 0.5=sublinear)
/// 
/// # Returns
/// Dendritic output value
#[inline(always)]
pub fn compute_dendrite_output(dendrite_sum: f32, exponent: f32) -> f32 {
    // Safety: Handle edge cases
    if dendrite_sum == 0.0 {
        return 0.0;
    }
    
    // For negative values, preserve sign and apply exponent to absolute value
    let sign = dendrite_sum.signum();
    let abs_sum = dendrite_sum.abs();
    
    // Apply exponent: sum^exponent
    // Use powf for flexibility, but optimize common cases
    let result = if exponent == 1.0 {
        abs_sum
    } else if exponent == 2.0 {
        abs_sum * abs_sum
    } else if exponent == 0.5 {
        abs_sum.sqrt()
    } else {
        abs_sum.powf(exponent)
    };
    
    sign * result
}

/// Group connections by dendrite and compute dendrite outputs
/// 
/// This function groups connections targeting a neuron by their dendrite_id,
/// computes the sum of inputs for each dendrite, then applies non-linear
/// integration to get the dendrite output.
/// 
/// # Arguments
/// * `connections` - Slice of connections targeting the neuron
/// * `input_values` - Pre-computed input values for each connection (parallel array)
/// * `weight_scale` - Optional weight scaling factor (for homeostatic plasticity)
/// * `params` - Simulation parameters
/// 
/// # Returns
/// Sum of all dendritic outputs
#[inline]
pub fn compute_neuron_input_from_dendrites(
    connections: &[Connection],
    input_values: &[f32],
    weight_scale: f32,
    params: &Params,
) -> f32 {
    // Safety check: arrays must be same length
    if connections.len() != input_values.len() {
        return 0.0;
    }
    
    if !params.dendritic_enabled {
        // Fallback to linear summation if dendritic computation is disabled
        return connections.iter()
            .zip(input_values.iter())
            .map(|(conn, input)| *input * conn.weight * weight_scale)
            .sum();
    }
    
    // Group connections by dendrite_id and compute weighted sums
    // Use HashMap for O(1) lookup performance (most neurons have few dendrites, but this is faster)
    use std::collections::HashMap;
    let mut dendrite_sums: HashMap<u16, (f32, u8)> = HashMap::with_capacity(connections.len().min(16)); // (sum, receptor_type)
    
    for (conn, input_val) in connections.iter().zip(input_values.iter()) {
        let weighted_input = *input_val * conn.weight * weight_scale;
        
        // Use HashMap for O(1) lookup and update
        let entry = dendrite_sums.entry(conn.dendrite_id).or_insert((0.0, conn.receptor_type));
        entry.0 += weighted_input;
    }
    
    // Compute dendrite outputs and sum them
    let mut total_input = 0.0;
    
    for (_dendrite_id, (dendrite_sum, receptor_type)) in dendrite_sums {
        // Determine exponent based on receptor type if separation is enabled
        let exponent = if params.dendritic_separate_excitatory_inhibitory {
            if receptor_type == 1 {
                // Inhibitory: use inhibitory exponent (typically sublinear)
                params.dendritic_inhibitory_exponent
            } else {
                // Excitatory: use main exponent (typically quadratic)
                params.dendritic_exponent
            }
        } else {
            // Use same exponent for all
            params.dendritic_exponent
        };
        
        let dendrite_output = compute_dendrite_output(dendrite_sum, exponent);
        total_input += dendrite_output;
    }
    
    total_input
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genome_neurons::{Gene, NEURON};

    fn make_test_params() -> Params {
        let mut params = Params::default();
        params.dendritic_enabled = true;
        params.dendritic_exponent = 2.0;
        params.dendritic_inhibitory_exponent = 0.5;
        params.dendritic_separate_excitatory_inhibitory = true;
        params
    }

    #[test]
    fn test_compute_dendrite_output_linear() {
        assert_eq!(compute_dendrite_output(5.0, 1.0), 5.0);
        assert_eq!(compute_dendrite_output(-3.0, 1.0), -3.0);
        assert_eq!(compute_dendrite_output(0.0, 1.0), 0.0);
    }

    #[test]
    fn test_compute_dendrite_output_quadratic() {
        assert_eq!(compute_dendrite_output(3.0, 2.0), 9.0);
        assert_eq!(compute_dendrite_output(-2.0, 2.0), -4.0);
        assert_eq!(compute_dendrite_output(0.0, 2.0), 0.0);
    }

    #[test]
    fn test_compute_dendrite_output_sublinear() {
        let result = compute_dendrite_output(4.0, 0.5);
        assert!((result - 2.0).abs() < 0.0001);
        let result_neg = compute_dendrite_output(-4.0, 0.5);
        assert!((result_neg + 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_compute_neuron_input_from_dendrites_disabled() {
        let mut params = Params::default();
        params.dendritic_enabled = false;
        
        let gene = Gene {
            source_type: NEURON,
            source_num: 0,
            sink_type: NEURON,
            sink_num: 1,
            weight: 8192, // 1.0 in float
        };
        let conn = Connection::from_gene(&gene, 0);
        let connections = vec![conn];
        let input_values = vec![2.0];
        
        let result = compute_neuron_input_from_dendrites(&connections, &input_values, 1.0, &params);
        assert_eq!(result, 2.0); // Linear: 2.0 * 1.0 * 1.0 = 2.0
    }

    #[test]
    fn test_compute_neuron_input_from_dendrites_quadratic() {
        let params = make_test_params();
        
        let gene = Gene {
            source_type: NEURON,
            source_num: 0,
            sink_type: NEURON,
            sink_num: 1,
            weight: 8192, // 1.0 in float
        };
        let conn = Connection::from_gene(&gene, 0);
        let connections = vec![conn.clone(), conn.clone()]; // Two connections to same dendrite
        let input_values = vec![2.0, 2.0];
        
        let result = compute_neuron_input_from_dendrites(&connections, &input_values, 1.0, &params);
        // Sum: 2.0*1.0 + 2.0*1.0 = 4.0
        // Quadratic: 4.0^2 = 16.0
        assert!((result - 16.0).abs() < 0.0001);
    }

    #[test]
    fn test_compute_neuron_input_from_dendrites_multiple_dendrites() {
        let params = make_test_params();
        
        let gene = Gene {
            source_type: NEURON,
            source_num: 0,
            sink_type: NEURON,
            sink_num: 1,
            weight: 8192, // 1.0 in float
        };
        let conn1 = Connection::from_gene(&gene, 0); // Dendrite 0
        let conn2 = Connection::from_gene(&gene, 1); // Dendrite 1
        let connections = vec![conn1, conn2];
        let input_values = vec![2.0, 2.0];
        
        let result = compute_neuron_input_from_dendrites(&connections, &input_values, 1.0, &params);
        // Dendrite 0: 2.0*1.0 = 2.0, quadratic = 4.0
        // Dendrite 1: 2.0*1.0 = 2.0, quadratic = 4.0
        // Total: 4.0 + 4.0 = 8.0
        assert!((result - 8.0).abs() < 0.0001);
    }
}

