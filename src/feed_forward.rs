// feed_forward.rs - reads sensors, returns actions, for an individual

use crate::indiv::Indiv;
use crate::genome_neurons::{SENSOR, ACTION};
use crate::sensors_actions::NUM_ACTIONS;
use crate::get_sensor::get_sensor;
use crate::grid::Grid;
use crate::signals::Signals;
use crate::params::Params;
use crate::peeps::Peeps;

pub fn feed_forward(indiv: &mut Indiv, sim_step: u32, grid: &Grid, signals: &Signals, params: &Params, peeps: &Peeps) -> [f32; NUM_ACTIONS] {
    let mut action_levels = [0.0f32; NUM_ACTIONS];
    
    let mut neuron_accumulators = vec![0.0f32; indiv.nnet.neurons.len()];
    
    let mut neuron_outputs_computed = false;
    
    for conn in &indiv.nnet.connections {
        if conn.sink_type == ACTION && !neuron_outputs_computed {
            // Update and latch all neuron outputs
            for (neuron_idx, neuron) in indiv.nnet.neurons.iter_mut().enumerate() {
                if neuron.driven {
                    neuron.output = neuron_accumulators[neuron_idx].tanh();
                }
            }
            neuron_outputs_computed = true;
        }
        
        // Obtain the connection's input value
        let input_val = if conn.source_type == SENSOR {
            get_sensor(indiv, unsafe { std::mem::transmute(conn.source_num) }, sim_step, grid, signals, params, peeps)
        } else {
            let neuron_idx = conn.source_num as usize;
            if neuron_idx < indiv.nnet.neurons.len() {
                indiv.nnet.neurons[neuron_idx].output
            } else {
                0.0 // Out of bounds - shouldn't happen, but safe fallback
            }
        };
        
        // Weight the connection's value and add to accumulator
        if conn.sink_type == ACTION {
            let action_idx = conn.sink_num as usize;
            if action_idx < NUM_ACTIONS {
                action_levels[action_idx] += input_val * conn.weight_as_float();
            }
        } else {
            let neuron_idx = conn.sink_num as usize;
            if neuron_idx < neuron_accumulators.len() {
                neuron_accumulators[neuron_idx] += input_val * conn.weight_as_float();
            }
        }
    }
    
    action_levels
}

