// indiv.rs
// Indiv is the structure that represents one individual agent

use crate::basic_types::{Coord, Dir};
use crate::genome_neurons::{Genome, NeuralNet, create_wiring_from_genome};
use crate::sensors_actions::{Sensor, NUM_ACTIONS};
use crate::params::Params;

#[derive(Clone)]
pub struct Indiv {
    pub alive: bool,
    pub index: u16,
    pub loc: Coord,
    pub birth_loc: Coord,
    pub age: u32,
    pub genome: Genome,
    pub nnet: NeuralNet,
    pub responsiveness: f32,  // 0.0..1.0 (0 is like asleep)
    pub osc_period: u32,       // 2..4*p.stepsPerGeneration
    pub long_probe_dist: u32,  // distance for long forward probe for obstructions
    pub last_move_dir: Dir,    // direction of last movement
    pub challenge_bits: u32,   // modified when the indiv accomplishes some task
    // Metabolic energy tracking (Phase 2)
    pub metabolic_energy: f32,        // Current energy level (0.0 = death)
    pub metabolic_cost_accumulated: f32, // Total cost accumulated this generation
    pub spike_count: u32,             // Total number of neuron activations this generation
    pub spikes_this_step: u32,        // Number of spikes in current step (for per-step cost)
}

impl Indiv {
    pub fn new() -> Self {
        Indiv {
            alive: false,
            index: 0,
            loc: Coord::default(),
            birth_loc: Coord::default(),
            age: 0,
            genome: Vec::new(),
            nnet: NeuralNet {
                connections: Vec::new(),
                neurons: Vec::new(),
            },
            responsiveness: 0.5,
            osc_period: 34,
            long_probe_dist: 0,
            last_move_dir: Dir::default(),
            challenge_bits: 0,
            metabolic_energy: 300.0,  // Default to 300 (will be set properly in initialize())
            metabolic_cost_accumulated: 0.0,
            spike_count: 0,
            spikes_this_step: 0,
        }
    }

    pub fn initialize(&mut self, index: u16, loc: Coord, genome: Genome, params: &Params) {
        self.index = index;
        self.loc = loc;
        self.birth_loc = loc;
        self.age = 0;
        self.osc_period = 34;
        self.alive = true;
        self.last_move_dir = Dir::random8();
        self.responsiveness = 0.5;
        self.long_probe_dist = params.long_probe_distance;
        self.challenge_bits = 0;
        self.genome = genome;
        // Initialize metabolic energy to steps_per_generation
        // This ensures individuals have enough energy to survive the full generation
        // regardless of how many steps the user configures
        self.metabolic_energy = params.steps_per_generation as f32;
        self.metabolic_cost_accumulated = 0.0;
        self.spike_count = 0;
        self.spikes_this_step = 0;
        self.create_wiring_from_genome(params);
    }

    pub fn create_wiring_from_genome(&mut self, params: &Params) {
        create_wiring_from_genome(&mut self.nnet, &self.genome, params);
    }

    pub fn feed_forward(&mut self, sim_step: u32, grid: &crate::grid::Grid, signals: &crate::signals::Signals, params: &Params, peeps: &crate::peeps::Peeps) -> [f32; NUM_ACTIONS] {
        crate::feed_forward::feed_forward(self, sim_step, grid, signals, params, peeps)
    }

    pub fn get_sensor(&self, sensor: Sensor, sim_step: u32, grid: &crate::grid::Grid, signals: &crate::signals::Signals, params: &Params, peeps: &crate::peeps::Peeps) -> f32 {
        crate::get_sensor::get_sensor(self, sensor, sim_step, grid, signals, params, peeps)
    }
}

impl Default for Indiv {
    fn default() -> Self {
        Self::new()
    }
}

