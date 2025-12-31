// create_barrier.rs
// Generates barrier points in the grid

use crate::grid::{Grid, BARRIER};
use crate::basic_types::Coord;
use crate::params::Params;
use crate::random::random_uint_range;
use crate::grid::visit_neighborhood;

pub fn create_barrier(grid: &mut Grid, barrier_type: u32, params: &Params) {
        grid.set_barrier_locations(Vec::new());
        grid.set_barrier_centers(Vec::new());

        let mut barrier_locations = Vec::new();
        let mut barrier_centers = Vec::new();

        let mut draw_box = |barrier_locations: &mut Vec<Coord>, min_x: i16, min_y: i16, max_x: i16, max_y: i16| {
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    grid.set(Coord::new(x, y), BARRIER);
                    barrier_locations.push(Coord::new(x, y));
                }
            }
        };

        match barrier_type {
            0 => {
                // No barrier
                return;
            }

            // Vertical bar in constant location
            1 => {
                let min_x = (params.size_x / 2) as i16;
                let max_x = min_x + 1;
                let min_y = (params.size_y / 4) as i16;
                let max_y = min_y + (params.size_y / 2) as i16;

                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        grid.set(Coord::new(x, y), BARRIER);
                        barrier_locations.push(Coord::new(x, y));
                    }
                }
            }

            // Vertical bar in random location
            2 => {
                let min_x = random_uint_range(20, params.size_x as u32 - 20) as i16;
                let max_x = min_x + 1;
                let min_y = random_uint_range(20, (params.size_y / 2) as u32 - 20) as i16;
                let max_y = min_y + (params.size_y / 2) as i16;

                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        grid.set(Coord::new(x, y), BARRIER);
                        barrier_locations.push(Coord::new(x, y));
                    }
                }
            }

            // Five blocks staggered
            3 => {
                let block_size_x = 2i16;
                let block_size_y = (params.size_x / 3) as i16;

                let mut x0 = (params.size_x / 4) as i16 - block_size_x / 2;
                let mut y0 = (params.size_y / 4) as i16 - block_size_y / 2;
                let mut x1 = x0 + block_size_x;
                let mut y1 = y0 + block_size_y;

                draw_box(&mut barrier_locations, x0, y0, x1, y1);
                x0 += (params.size_x / 2) as i16;
                x1 = x0 + block_size_x;
                draw_box(&mut barrier_locations, x0, y0, x1, y1);
                y0 += (params.size_y / 2) as i16;
                y1 = y0 + block_size_y;
                draw_box(&mut barrier_locations, x0, y0, x1, y1);
                x0 -= (params.size_x / 2) as i16;
                x1 = x0 + block_size_x;
                draw_box(&mut barrier_locations, x0, y0, x1, y1);
                x0 = (params.size_x / 2) as i16 - block_size_x / 2;
                x1 = x0 + block_size_x;
                y0 = (params.size_y / 2) as i16 - block_size_y / 2;
                y1 = y0 + block_size_y;
                draw_box(&mut barrier_locations, x0, y0, x1, y1);
            }

            // Horizontal bar in constant location
            4 => {
                let min_x = (params.size_x / 4) as i16;
                let max_x = min_x + (params.size_x / 2) as i16;
                let min_y = (params.size_y / 2 + params.size_y / 4) as i16;
                let max_y = min_y + 2;

                for x in min_x..=max_x {
                    for y in min_y..=max_y {
                        grid.set(Coord::new(x, y), BARRIER);
                        barrier_locations.push(Coord::new(x, y));
                    }
                }
            }

            // Three floating islands
            5 => {
                let radius = 3.0f32;
                let margin = (2.0 * radius) as u32;

                let random_loc = || -> Coord {
                    Coord::new(
                        random_uint_range(margin, params.size_x as u32 - margin) as i16,
                        random_uint_range(margin, params.size_y as u32 - margin) as i16,
                    )
                };

                let center0 = random_loc();
                let mut center1;
                let mut center2;

                loop {
                    center1 = random_loc();
                    if (center0 - center1).length() >= margin as u32 {
                        break;
                    }
                }

                loop {
                    center2 = random_loc();
                    if (center0 - center2).length() >= margin as u32
                        && (center1 - center2).length() >= margin as u32
                    {
                        break;
                    }
                }

                barrier_centers.push(center0);

                visit_neighborhood(center0, radius, params, |loc| {
                    grid.set(loc, BARRIER);
                    barrier_locations.push(loc);
                });
            }

            // Spots
            6 => {
                let number_of_locations = 5u32;
                let radius = 5.0f32;

                let vertical_slice_size = params.size_y as u32 / (number_of_locations + 1);

                for n in 1..=number_of_locations {
                    let loc = Coord::new(
                        (params.size_x / 2) as i16,
                        (n * vertical_slice_size) as i16,
                    );
                    visit_neighborhood(loc, radius, params, |tloc| {
                        grid.set(tloc, BARRIER);
                        barrier_locations.push(tloc);
                    });
                    barrier_centers.push(loc);
                }
            }

            _ => {
                panic!("Unknown barrier type: {}", barrier_type);
            }
        }

        grid.set_barrier_locations(barrier_locations);
        grid.set_barrier_centers(barrier_centers);
    }

