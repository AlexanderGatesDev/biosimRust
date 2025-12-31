// signals.rs
// Container for pheromones

use crate::basic_types::Coord;
use crate::grid::visit_neighborhood;
use crate::params::Params;

pub const SIGNAL_MIN: u8 = 0;
pub const SIGNAL_MAX: u8 = u8::MAX;

pub struct Signals {
    data: Vec<Vec<Vec<u8>>>,  // [layer][x][y]
}

impl Signals {
    pub fn new() -> Self {
        Signals { data: Vec::new() }
    }

    pub fn init(&mut self, num_layers: u16, size_x: u16, size_y: u16) {
        self.data = (0..num_layers)
            .map(|_| {
                (0..size_x)
                    .map(|_| vec![0u8; size_y as usize])
                    .collect()
            })
            .collect();
    }

    pub fn get_magnitude(&self, layer_num: u16, loc: Coord) -> u8 {
        if layer_num as usize >= self.data.len()
            || loc.x < 0
            || loc.x as usize >= self.data[layer_num as usize].len()
            || loc.y < 0
            || loc.y as usize >= self.data[layer_num as usize][loc.x as usize].len()
        {
            return 0;
        }
        self.data[layer_num as usize][loc.x as usize][loc.y as usize]
    }

    pub fn increment(&mut self, layer_num: u16, loc: Coord, params: &Params) {
        const RADIUS: f32 = 1.5;
        const CENTER_INCREASE_AMOUNT: u8 = 2;
        const NEIGHBOR_INCREASE_AMOUNT: u8 = 1;

        // Increment neighbors
        visit_neighborhood(loc, RADIUS, params, |tloc| {
            if let Some(val) = self.get_mut(layer_num, tloc) {
                if *val < SIGNAL_MAX {
                    *val = (*val as u16 + NEIGHBOR_INCREASE_AMOUNT as u16).min(SIGNAL_MAX as u16) as u8;
                }
            }
        });

        // Increment center
        if let Some(val) = self.get_mut(layer_num, loc) {
            if *val < SIGNAL_MAX {
                *val = (*val as u16 + CENTER_INCREASE_AMOUNT as u16).min(SIGNAL_MAX as u16) as u8;
            }
        }
    }

    fn get_mut(&mut self, layer_num: u16, loc: Coord) -> Option<&mut u8> {
        if layer_num as usize >= self.data.len()
            || loc.x < 0
            || loc.x as usize >= self.data[layer_num as usize].len()
            || loc.y < 0
            || loc.y as usize >= self.data[layer_num as usize][loc.x as usize].len()
        {
            return None;
        }
        Some(&mut self.data[layer_num as usize][loc.x as usize][loc.y as usize])
    }

    pub fn zero_fill(&mut self) {
        for layer in &mut self.data {
            for col in layer {
                for val in col {
                    *val = 0;
                }
            }
        }
    }

    pub fn fade(&mut self, layer_num: usize, params: &Params) {
        const FADE_AMOUNT: u8 = 1;

        if layer_num >= self.data.len() {
            return;
        }

        for x in 0..params.size_x as usize {
            for y in 0..params.size_y as usize {
                if x < self.data[layer_num].len() && y < self.data[layer_num][x].len() {
                    if self.data[layer_num][x][y] >= FADE_AMOUNT {
                        self.data[layer_num][x][y] -= FADE_AMOUNT;
                    } else {
                        self.data[layer_num][x][y] = 0;
                    }
                }
            }
        }
    }
}

impl Default for Signals {
    fn default() -> Self {
        Self::new()
    }
}

