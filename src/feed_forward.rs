// feed_forward.rs - reads sensors, returns actions, for an individual

use crate::indiv::Indiv;
use crate::genome_neurons::{SENSOR, ACTION, Connection};
use crate::sensors_actions::NUM_ACTIONS;
use crate::get_sensor::get_sensor;
use crate::grid::Grid;
use crate::signals::Signals;
use crate::params::Params;
use crate::peeps::Peeps;
use crate::dendritic::compute_neuron_input_from_dendrites;
use std::collections::HashMap;

pub fn feed_forward(indiv: &mut Indiv, sim_step: u32, grid: &Grid, signals: &Signals, params: &Params, peeps: &Peeps) -> [f32; NUM_ACTIONS] {
    let mut action_levels = [0.0f32; NUM_ACTIONS];
    
    let num_neurons = indiv.nnet.neurons.len();
    let mut neuron_accumulators = vec![0.0f32; num_neurons];
    
    // Phase 4.3: If dendritic computation is enabled, we need to group connections
    // by target neuron and dendrite_id, then apply non-linear integration
    if params.dendritic_enabled {
        // Separate connections into neuron-targeting and action-targeting
        let mut neuron_connections: Vec<(&Connection, usize)> = Vec::new();
        let mut action_connections: Vec<&Connection> = Vec::new();
        
        for (_idx, conn) in indiv.nnet.connections.iter().enumerate() {
            if conn.sink_type == ACTION {
                action_connections.push(conn);
            } else {
                // Safety check: ensure sink_num is valid
                let neuron_idx = conn.sink_num as usize;
                if neuron_idx < num_neurons {
                    neuron_connections.push((conn, neuron_idx));
                }
            }
        }
        
        // Group neuron connections by target neuron
        // Store raw input values (without weight scaling) - dendritic function will handle weights
        let mut connections_by_neuron: HashMap<usize, Vec<(&Connection, f32)>> = HashMap::new();
        
        for (conn, neuron_idx) in neuron_connections {
            // Compute raw input value for this connection (without weight)
            let input_val = if conn.source_type == SENSOR {
                get_sensor(indiv, unsafe { std::mem::transmute(conn.source_num) }, sim_step, grid, signals, params, peeps)
            } else {
                let source_neuron_idx = conn.source_num as usize;
                if source_neuron_idx < num_neurons {
                    indiv.nnet.neurons[source_neuron_idx].output
                } else {
                    0.0 // Safety fallback
                }
            };
            
            connections_by_neuron
                .entry(neuron_idx)
                .or_insert_with(Vec::new)
                .push((conn, input_val));
        }
        
        // Compute dendritic integration for each neuron
        // Pre-allocate capacity for better performance
        for (neuron_idx, conn_inputs) in connections_by_neuron {
            // Separate connections and input values for dendritic computation
            // Clone connections only when necessary (dendritic computation needs owned values)
            let num_conns = conn_inputs.len();
            let mut connections = Vec::with_capacity(num_conns);
            let mut input_values = Vec::with_capacity(num_conns);
            
            for (conn, input_val) in conn_inputs {
                connections.push((*conn).clone());
                input_values.push(input_val);
            }
            
            // Get homeostatic scaling factor if enabled
            let weight_scale = if params.homeostasis_enabled {
                indiv.nnet.neurons[neuron_idx].homeostatic_scale
            } else {
                1.0
            };
            
            // Compute neuron input using dendritic integration
            let neuron_input = compute_neuron_input_from_dendrites(&connections, &input_values, weight_scale, params);
            neuron_accumulators[neuron_idx] += neuron_input;
        }
        
        // Process action connections (linear summation)
        for conn in action_connections {
            let input_val = if conn.source_type == SENSOR {
                get_sensor(indiv, unsafe { std::mem::transmute(conn.source_num) }, sim_step, grid, signals, params, peeps)
            } else {
                let neuron_idx = conn.source_num as usize;
                if neuron_idx < num_neurons {
                    indiv.nnet.neurons[neuron_idx].output
                } else {
                    0.0 // Safety fallback
                }
            };
            
            let action_idx = conn.sink_num as usize;
            if action_idx < NUM_ACTIONS {
                action_levels[action_idx] += input_val * conn.weight;
            }
        }
    } else {
        // Original linear computation (faster path when dendritic computation is disabled)
        let mut neuron_outputs_computed = false;
        
        for conn in &indiv.nnet.connections {
            if conn.sink_type == ACTION && !neuron_outputs_computed {
                // Update and latch all neuron outputs
                for (neuron_idx, neuron) in indiv.nnet.neurons.iter_mut().enumerate() {
                    if neuron.driven {
                        let output = neuron_accumulators[neuron_idx].tanh();
                        neuron.output = output;
                        // Track spikes: count neuron activations (output > threshold)
                        // This is used for dynamic metabolic cost calculation
                        if output.abs() > 0.1 {
                            indiv.spike_count += 1;
                            indiv.spikes_this_step += 1;
                        }
                    }
                }
                neuron_outputs_computed = true;
            }
            
            // Obtain the connection's input value
            let input_val = if conn.source_type == SENSOR {
                get_sensor(indiv, unsafe { std::mem::transmute(conn.source_num) }, sim_step, grid, signals, params, peeps)
            } else {
                let neuron_idx = conn.source_num as usize;
                if neuron_idx < num_neurons {
                    indiv.nnet.neurons[neuron_idx].output
                } else {
                    0.0 // Out of bounds - shouldn't happen, but safe fallback
                }
            };
            
            // Weight the connection's value and add to accumulator
            // Phase 4: Connection now uses f32 weight directly (no conversion needed)
            // Apply homeostatic scaling for connections targeting neurons (Phase 3)
            if conn.sink_type == ACTION {
                let action_idx = conn.sink_num as usize;
                if action_idx < NUM_ACTIONS {
                    action_levels[action_idx] += input_val * conn.weight;
                }
            } else {
                let neuron_idx = conn.sink_num as usize;
                if neuron_idx < neuron_accumulators.len() {
                    // Apply homeostatic scaling if enabled
                    let scaled_weight = if params.homeostasis_enabled && neuron_idx < num_neurons {
                        conn.weight * indiv.nnet.neurons[neuron_idx].homeostatic_scale
                    } else {
                        conn.weight
                    };
                    neuron_accumulators[neuron_idx] += input_val * scaled_weight;
                }
            }
        }
    }
    
    // Update and latch all neuron outputs (after all inputs are computed)
    for (neuron_idx, neuron) in indiv.nnet.neurons.iter_mut().enumerate() {
        if neuron.driven {
            let output = neuron_accumulators[neuron_idx].tanh();
            neuron.output = output;
            // Track spikes: count neuron activations (output > threshold)
            if output.abs() > 0.1 {
                indiv.spike_count += 1;
                indiv.spikes_this_step += 1;
            }
        }
    }
    
    action_levels
}

