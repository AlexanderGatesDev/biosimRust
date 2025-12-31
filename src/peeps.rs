// peeps.rs
// Manages a container of individual agents of type Indiv and their
// locations in the grid container

use crate::basic_types::Coord;
use crate::grid::{Grid, EMPTY};
use crate::indiv::Indiv;
use std::sync::{Arc, Mutex};

pub struct Peeps {
    pub individuals: Vec<Indiv>,  // Index value 0 is reserved (public for parallel access)
    death_queue: Arc<Mutex<Vec<u16>>>,
    move_queue: Arc<Mutex<Vec<(u16, Coord)>>>,
}

impl Peeps {
    pub fn new() -> Self {
        Peeps {
            individuals: Vec::new(),
            death_queue: Arc::new(Mutex::new(Vec::new())),
            move_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn init(&mut self, population: u32) {
        // Index 0 is reserved, so add one:
        self.individuals.resize((population + 1) as usize, Indiv::new());
    }

    // Safe to call during multithread mode.
    // Indiv will remain alive and in-world until end of sim step when
    // drain_death_queue() is called.
    pub fn queue_for_death(&self, indiv: &Indiv) {
        assert!(indiv.alive);
        if let Ok(mut queue) = self.death_queue.lock() {
            queue.push(indiv.index);
        }
    }

    // Called in single-thread mode at end of sim step.
    pub fn drain_death_queue(&mut self, grid: &mut Grid) {
        if let Ok(mut queue) = self.death_queue.lock() {
            for index in queue.iter() {
                let indiv = &mut self.individuals[*index as usize];
                grid.set(indiv.loc, EMPTY);
                indiv.alive = false;
            }
            queue.clear();
        }
    }

    // Safe to call during multithread mode.
    pub fn queue_for_move(&self, indiv: &Indiv, new_loc: Coord) {
        assert!(indiv.alive);
        if let Ok(mut queue) = self.move_queue.lock() {
            queue.push((indiv.index, new_loc));
        }
    }

    // Called in single-thread mode at end of sim step.
    pub fn drain_move_queue(&mut self, grid: &mut Grid) {
        if let Ok(mut queue) = self.move_queue.lock() {
            for (index, new_loc) in queue.iter() {
                let indiv = &mut self.individuals[*index as usize];
                if indiv.alive {
                    if grid.is_empty_at(*new_loc) {
                        let old_loc = indiv.loc;
                        grid.set(old_loc, EMPTY);
                        grid.set(*new_loc, indiv.index);
                        indiv.loc = *new_loc;
                        // Calculate direction of movement BEFORE updating loc
                        let move_offset = *new_loc - old_loc;
                        indiv.last_move_dir = move_offset.as_dir();
                    }
                }
            }
            queue.clear();
        }
    }

    pub fn death_queue_size(&self) -> usize {
        if let Ok(queue) = self.death_queue.lock() {
            queue.len()
        } else {
            0
        }
    }

    pub fn get_indiv(&self, loc: Coord, grid: &Grid) -> Option<&Indiv> {
        let index = grid.at(loc);
        if index == 0 || index as usize >= self.individuals.len() {
            return None;
        }
        Some(&self.individuals[index as usize])
    }

    pub fn get_indiv_mut(&mut self, loc: Coord, grid: &Grid) -> Option<&mut Indiv> {
        let index = grid.at(loc);
        if index == 0 || index as usize >= self.individuals.len() {
            return None;
        }
        Some(&mut self.individuals[index as usize])
    }

    pub fn get(&self, index: u16) -> Option<&Indiv> {
        if index == 0 || index as usize >= self.individuals.len() {
            return None;
        }
        Some(&self.individuals[index as usize])
    }

    pub fn get_mut(&mut self, index: u16) -> Option<&mut Indiv> {
        if index == 0 || index as usize >= self.individuals.len() {
            return None;
        }
        Some(&mut self.individuals[index as usize])
    }
}

impl Default for Peeps {
    fn default() -> Self {
        Self::new()
    }
}


