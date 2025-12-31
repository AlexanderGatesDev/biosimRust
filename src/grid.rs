// grid.rs
// The grid is the 2D arena where the agents live

use crate::basic_types::Coord;
use crate::params::Params;
use crate::random::random_uint_range;

pub const EMPTY: u16 = 0;  // Index value 0 is reserved
pub const BARRIER: u16 = 0xffff;

pub struct Grid {
    data: Vec<Vec<u16>>,  // [x][y]
    barrier_locations: Vec<Coord>,
    barrier_centers: Vec<Coord>,
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            data: Vec::new(),
            barrier_locations: Vec::new(),
            barrier_centers: Vec::new(),
        }
    }

    pub fn init(&mut self, size_x: u16, size_y: u16) {
        self.data = (0..size_x)
            .map(|_| vec![0u16; size_y as usize])
            .collect();
    }

    pub fn zero_fill(&mut self) {
        for col in &mut self.data {
            for val in col {
                *val = 0;
            }
        }
    }

    pub fn size_x(&self) -> u16 {
        self.data.len() as u16
    }

    pub fn size_y(&self) -> u16 {
        if self.data.is_empty() {
            return 0;
        }
        self.data[0].len() as u16
    }

    pub fn is_in_bounds(&self, loc: Coord) -> bool {
        if self.data.is_empty() {
            return false;
        }
        loc.x >= 0
            && (loc.x as usize) < self.data.len()
            && loc.y >= 0
            && (loc.y as usize) < self.data[0].len()
    }

    pub fn is_empty_at(&self, loc: Coord) -> bool {
        self.at(loc) == EMPTY
    }

    pub fn is_barrier_at(&self, loc: Coord) -> bool {
        self.at(loc) == BARRIER
    }

    pub fn is_occupied_at(&self, loc: Coord) -> bool {
        let val = self.at(loc);
        val != EMPTY && val != BARRIER
    }

    pub fn is_border(&self, loc: Coord) -> bool {
        loc.x == 0
            || loc.x == (self.size_x() - 1) as i16
            || loc.y == 0
            || loc.y == (self.size_y() - 1) as i16
    }

    pub fn at(&self, loc: Coord) -> u16 {
        if !self.is_in_bounds(loc) {
            return EMPTY;
        }
        self.data[loc.x as usize][loc.y as usize]
    }

    pub fn at_xy(&self, x: u16, y: u16) -> u16 {
        if x as usize >= self.data.len() || y as usize >= self.data[0].len() {
            return EMPTY;
        }
        self.data[x as usize][y as usize]
    }

    pub fn set(&mut self, loc: Coord, val: u16) {
        if self.is_in_bounds(loc) {
            self.data[loc.x as usize][loc.y as usize] = val;
        }
    }

    pub fn set_xy(&mut self, x: u16, y: u16, val: u16) {
        if (x as usize) < self.data.len() && (y as usize) < self.data[0].len() {
            self.data[x as usize][y as usize] = val;
        }
    }

    pub fn find_empty_location(&self, params: &Params) -> Coord {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 1000;
        loop {
            attempts += 1;
            if attempts > MAX_ATTEMPTS {
                for y in 0..params.size_y as i16 {
                    for x in 0..params.size_x as i16 {
                        let loc = Coord { x, y };
                        if self.is_empty_at(loc) {
                            return loc;
                        }
                    }
                }
                // If systematic search also fails, grid is full
                // Count how many cells are actually empty/barrier/occupied for debugging
                let mut empty_count = 0;
                let mut barrier_count = 0;
                let mut occupied_count = 0;
                for y in 0..params.size_y as i16 {
                    for x in 0..params.size_x as i16 {
                        let loc = Coord { x, y };
                        let val = self.at(loc);
                        if val == EMPTY {
                            empty_count += 1;
                        } else if val == BARRIER {
                            barrier_count += 1;
                        } else {
                            occupied_count += 1;
                        }
                    }
                }
                eprintln!("ERROR: find_empty_location failed! Grid stats - empty: {}, barriers: {}, occupied: {}, total: {}", 
                         empty_count, barrier_count, occupied_count, params.size_x as u32 * params.size_y as u32);
                // Last resort: return a location anyway (may be occupied, but prevents infinite loop)
                return Coord {
                    x: random_uint_range(0, params.size_x as u32 - 1) as i16,
                    y: random_uint_range(0, params.size_y as u32 - 1) as i16,
                };
            }
            let x = random_uint_range(0, params.size_x as u32 - 1) as i16;
            let y = random_uint_range(0, params.size_y as u32 - 1) as i16;
            let loc = Coord { x, y };
            
            if !self.is_in_bounds(loc) {
                continue; // Skip invalid coordinates
            }
            
            if self.is_empty_at(loc) {
                return loc;
            }
        }
    }

    pub fn get_barrier_locations(&self) -> &[Coord] {
        &self.barrier_locations
    }

    pub fn get_barrier_centers(&self) -> &[Coord] {
        &self.barrier_centers
    }

    pub fn set_barrier_locations(&mut self, locations: Vec<Coord>) {
        self.barrier_locations = locations;
    }

    pub fn set_barrier_centers(&mut self, centers: Vec<Coord>) {
        self.barrier_centers = centers;
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

pub fn visit_neighborhood<F>(loc: Coord, radius: f32, params: &Params, mut f: F)
where
    F: FnMut(Coord),
{
    let radius_int = radius as i32;
    let min_dx = (-radius_int).max(-(loc.x as i32));
    let max_dx = radius_int.min((params.size_x as i32) - (loc.x as i32) - 1);
    
    for dx in min_dx..=max_dx {
        let x = (loc.x as i32 + dx) as i16;
        if x < 0 || x as u16 >= params.size_x {
            continue;
        }
        
        let extent_y = ((radius * radius - (dx * dx) as f32).sqrt()) as i32;
        let min_dy = (-extent_y).max(-(loc.y as i32));
        let max_dy = extent_y.min((params.size_y as i32) - (loc.y as i32) - 1);
        
        for dy in min_dy..=max_dy {
            let y = (loc.y as i32 + dy) as i16;
            if y < 0 || y as u16 >= params.size_y {
                continue;
            }
            f(Coord { x, y });
        }
    }
}

