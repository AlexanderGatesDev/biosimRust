// get_sensor.rs
// Full sensor value calculations

use crate::indiv::Indiv;
use crate::sensors_actions::Sensor;
use crate::random::random_uint;
use crate::random::RANDOM_UINT_MAX;
use crate::grid::{Grid, visit_neighborhood};
use crate::signals::{Signals, SIGNAL_MAX};
use crate::params::Params;
use crate::peeps::Peeps;
use crate::basic_types::{Coord, Dir, Compass};
use crate::genome_compare::genome_similarity as genome_similarity_with_params;

// Helper function: Get population density along a specific axis
fn get_population_density_along_axis(
    loc: Coord,
    dir: Dir,
    grid: &Grid,
    params: &Params,
) -> f32 {
    // Require a defined axis
    if dir.as_int() == Compass::Center as u8 {
        return 0.5;
    }

    let mut sum = 0.0f64;
    let dir_vec = dir.as_normalized_coord();
    let len = (dir_vec.x as f64 * dir_vec.x as f64 + dir_vec.y as f64 * dir_vec.y as f64).sqrt();
    let dir_vec_x = dir_vec.x as f64 / len;
    let dir_vec_y = dir_vec.y as f64 / len; // Unit vector components along dir

    visit_neighborhood(loc, params.population_sensor_radius, params, |tloc| {
        if tloc != loc && grid.is_occupied_at(tloc) {
            let offset = tloc - loc;
            let proj = dir_vec_x * offset.x as f64 + dir_vec_y * offset.y as f64; // Magnitude of projection along dir
            let contrib = proj / (offset.x as f64 * offset.x as f64 + offset.y as f64 * offset.y as f64);
            sum += contrib;
        }
    });

    let max_sum_mag = 6.0 * params.population_sensor_radius as f64;
    
    let mut sensor_val = sum / max_sum_mag; // convert to -1.0..1.0
    sensor_val = (sensor_val + 1.0) / 2.0; // convert to 0.0..1.0
    
    sensor_val as f32
}

// Helper function: Get short probe barrier distance
fn get_short_probe_barrier_distance(
    loc0: Coord,
    dir: Dir,
    probe_distance: u32,
    grid: &Grid,
) -> f32 {
    let mut count_fwd = 0u32;
    let mut count_rev = 0u32;
    let mut loc = loc0 + dir;
    let mut num_locs_to_test = probe_distance;
    
    // Scan positive direction
    while num_locs_to_test > 0 && grid.is_in_bounds(loc) && !grid.is_barrier_at(loc) {
        count_fwd += 1;
        loc = loc + dir;
        num_locs_to_test -= 1;
    }
    if num_locs_to_test > 0 && !grid.is_in_bounds(loc) {
        count_fwd = probe_distance;
    }
    
    // Scan negative direction
    num_locs_to_test = probe_distance;
    loc = loc0 - dir;
    while num_locs_to_test > 0 && grid.is_in_bounds(loc) && !grid.is_barrier_at(loc) {
        count_rev += 1;
        loc = loc - dir;
        num_locs_to_test -= 1;
    }
    if num_locs_to_test > 0 && !grid.is_in_bounds(loc) {
        count_rev = probe_distance;
    }

    let mut sensor_val = (count_fwd as f32 - count_rev as f32) + probe_distance as f32; // convert to 0..2*probeDistance
    sensor_val = (sensor_val / 2.0) / probe_distance as f32; // convert to 0.0..1.0
    sensor_val
}

// Helper function: Get signal density in neighborhood
fn get_signal_density(
    layer_num: u16,
    loc: Coord,
    signals: &Signals,
    params: &Params,
) -> f32 {
    let mut count_locs = 0u32;
    let mut sum = 0u64;
    let center = loc;

    visit_neighborhood(center, params.signal_sensor_radius, params, |tloc| {
        count_locs += 1;
        sum += signals.get_magnitude(layer_num, tloc) as u64;
    });
    
    let max_sum = count_locs as f64 * SIGNAL_MAX as f64;
    let sensor_val = sum as f64 / max_sum; // convert to 0.0..1.0
    
    sensor_val as f32
}

// Helper function: Get signal density along axis
fn get_signal_density_along_axis(
    layer_num: u16,
    loc: Coord,
    dir: Dir,
    signals: &Signals,
    params: &Params,
) -> f32 {
    // Require a defined axis
    if dir.as_int() == Compass::Center as u8 {
        return 0.5;
    }

    let mut sum = 0.0f64;
    let dir_vec = dir.as_normalized_coord();
    let len = (dir_vec.x as f64 * dir_vec.x as f64 + dir_vec.y as f64 * dir_vec.y as f64).sqrt();
    let dir_vec_x = dir_vec.x as f64 / len;
    let dir_vec_y = dir_vec.y as f64 / len; // Unit vector components along dir

    visit_neighborhood(loc, params.signal_sensor_radius, params, |tloc| {
        if tloc != loc {
            let offset = tloc - loc;
            let proj = dir_vec_x * offset.x as f64 + dir_vec_y * offset.y as f64; // Magnitude of projection along dir
            let contrib = (proj * signals.get_magnitude(layer_num, tloc) as f64) /
                (offset.x as f64 * offset.x as f64 + offset.y as f64 * offset.y as f64);
            sum += contrib;
        }
    });

    let max_sum_mag = 6.0 * params.signal_sensor_radius as f64 * SIGNAL_MAX as f64;
    let mut sensor_val = sum / max_sum_mag; // convert to -1.0..1.0
    sensor_val = (sensor_val + 1.0) / 2.0; // convert to 0.0..1.0
    
    sensor_val as f32
}

// Helper function: Long probe population forward
fn long_probe_population_fwd(
    loc: Coord,
    dir: Dir,
    long_probe_dist: u32,
    grid: &Grid,
) -> u32 {
    assert!(long_probe_dist > 0);
    let mut count = 0u32;
    let mut test_loc = loc + dir;
    let mut num_locs_to_test = long_probe_dist;
    
    while num_locs_to_test > 0 && grid.is_in_bounds(test_loc) && grid.is_empty_at(test_loc) {
        count += 1;
        test_loc = test_loc + dir;
        num_locs_to_test -= 1;
    }
    
    if num_locs_to_test > 0 && (!grid.is_in_bounds(test_loc) || grid.is_barrier_at(test_loc)) {
        long_probe_dist
    } else {
        count
    }
}

// Helper function: Long probe barrier forward
fn long_probe_barrier_fwd(
    loc: Coord,
    dir: Dir,
    long_probe_dist: u32,
    grid: &Grid,
) -> u32 {
    assert!(long_probe_dist > 0);
    let mut count = 0u32;
    let mut test_loc = loc + dir;
    let mut num_locs_to_test = long_probe_dist;
    
    while num_locs_to_test > 0 && grid.is_in_bounds(test_loc) && !grid.is_barrier_at(test_loc) {
        count += 1;
        test_loc = test_loc + dir;
        num_locs_to_test -= 1;
    }
    
    if num_locs_to_test > 0 && !grid.is_in_bounds(test_loc) {
        long_probe_dist
    } else {
        count
    }
}

// Main sensor function
pub fn get_sensor(
    indiv: &Indiv,
    sensor: Sensor,
    sim_step: u32,
    grid: &Grid,
    signals: &Signals,
    params: &Params,
    peeps: &Peeps,
) -> f32 {
    let mut sensor_val = 0.0f32;

    match sensor {
        Sensor::Age => {
            // Converts age (units of simSteps compared to life expectancy)
            // linearly to normalized sensor range 0.0..1.0
            sensor_val = indiv.age as f32 / params.steps_per_generation as f32;
        }

        Sensor::BoundaryDist => {
            // Finds closest boundary, compares that to the max possible dist
            // to a boundary from the center, and converts that linearly to the
            // sensor range 0.0..1.0
            let dist_x = (indiv.loc.x as i32).min((params.size_x as i32 - indiv.loc.x as i32) - 1);
            let dist_y = (indiv.loc.y as i32).min((params.size_y as i32 - indiv.loc.y as i32) - 1);
            let closest = dist_x.min(dist_y);
            let max_possible = (params.size_x as i32 / 2 - 1).max(params.size_y as i32 / 2 - 1);
            sensor_val = closest as f32 / max_possible as f32;
        }

        Sensor::BoundaryDistX => {
            // Measures the distance to nearest boundary in the east-west axis,
            // max distance is half the grid width; scaled to sensor range 0.0..1.0.
            let min_dist_x = (indiv.loc.x as i32).min((params.size_x as i32 - indiv.loc.x as i32) - 1);
            sensor_val = min_dist_x as f32 / (params.size_x as f32 / 2.0);
        }

        Sensor::BoundaryDistY => {
            // Measures the distance to nearest boundary in the south-north axis,
            // max distance is half the grid height; scaled to sensor range 0.0..1.0.
            let min_dist_y = (indiv.loc.y as i32).min((params.size_y as i32 - indiv.loc.y as i32) - 1);
            sensor_val = min_dist_y as f32 / (params.size_y as f32 / 2.0);
        }

        Sensor::LastMoveDirX => {
            // X component -1,0,1 maps to sensor values 0.0, 0.5, 1.0
            let last_x = indiv.last_move_dir.as_normalized_coord().x;
            sensor_val = if last_x == 0 {
                0.5
            } else if last_x == -1 {
                0.0
            } else {
                1.0
            };
        }

        Sensor::LastMoveDirY => {
            // Y component -1,0,1 maps to sensor values 0.0, 0.5, 1.0
            let last_y = indiv.last_move_dir.as_normalized_coord().y;
            sensor_val = if last_y == 0 {
                0.5
            } else if last_y == -1 {
                0.0
            } else {
                1.0
            };
        }

        Sensor::LocX => {
            // Maps current X location 0..p.sizeX-1 to sensor range 0.0..1.0
            sensor_val = indiv.loc.x as f32 / (params.size_x as f32 - 1.0);
        }

        Sensor::LocY => {
            // Maps current Y location 0..p.sizeY-1 to sensor range 0.0..1.0
            sensor_val = indiv.loc.y as f32 / (params.size_y as f32 - 1.0);
        }

        Sensor::Osc1 => {
            // Maps the oscillator sine wave to sensor range 0.0..1.0;
            // cycles starts at simStep 0 for everybody.
            let phase = (sim_step % indiv.osc_period) as f32 / indiv.osc_period as f32; // 0.0..1.0
            let mut factor = -(phase * 2.0 * std::f32::consts::PI).cos();
            factor += 1.0;    // convert to 0.0..2.0
            factor /= 2.0;     // convert to 0.0..1.0
            sensor_val = factor;
            // Clip any round-off error
            sensor_val = sensor_val.min(1.0).max(0.0);
        }

        Sensor::LongprobePopFwd => {
            // Measures the distance to the nearest other individual in the
            // forward direction. If none found, returns the maximum sensor value.
            // Maps the result to the sensor range 0.0..1.0.
            let count = long_probe_population_fwd(indiv.loc, indiv.last_move_dir, indiv.long_probe_dist, grid);
            sensor_val = count as f32 / indiv.long_probe_dist as f32; // 0..1
        }

        Sensor::LongprobeBarFwd => {
            // Measures the distance to the nearest barrier in the forward
            // direction. If none found, returns the maximum sensor value.
            // Maps the result to the sensor range 0.0..1.0.
            let count = long_probe_barrier_fwd(indiv.loc, indiv.last_move_dir, indiv.long_probe_dist, grid);
            sensor_val = count as f32 / indiv.long_probe_dist as f32; // 0..1
        }

        Sensor::Population => {
            // Returns population density in neighborhood converted linearly from
            // 0..100% to sensor range
            let mut count_locs = 0u32;
            let mut count_occupied = 0u32;
            let center = indiv.loc;

            visit_neighborhood(center, params.population_sensor_radius, params, |tloc| {
                count_locs += 1;
                if grid.is_occupied_at(tloc) {
                    count_occupied += 1;
                }
            });
            
            if count_locs > 0 {
                sensor_val = count_occupied as f32 / count_locs as f32;
            } else {
                sensor_val = 0.0;
            }
        }

        Sensor::PopulationFwd => {
            // Sense population density along axis of last movement direction, mapped
            // to sensor range 0.0..1.0
            sensor_val = get_population_density_along_axis(indiv.loc, indiv.last_move_dir, grid, params);
        }

        Sensor::PopulationLr => {
            // Sense population density along an axis 90 degrees from last movement direction
            sensor_val = get_population_density_along_axis(indiv.loc, indiv.last_move_dir.rotate90_deg_cw(), grid, params);
        }

        Sensor::BarrierFwd => {
            // Sense the nearest barrier along axis of last movement direction, mapped
            // to sensor range 0.0..1.0
            sensor_val = get_short_probe_barrier_distance(
                indiv.loc,
                indiv.last_move_dir,
                params.short_probe_barrier_distance,
                grid,
            );
        }

        Sensor::BarrierLr => {
            // Sense the nearest barrier along axis perpendicular to last movement direction, mapped
            // to sensor range 0.0..1.0
            sensor_val = get_short_probe_barrier_distance(
                indiv.loc,
                indiv.last_move_dir.rotate90_deg_cw(),
                params.short_probe_barrier_distance,
                grid,
            );
        }

        Sensor::Random => {
            // Returns a random sensor value in the range 0.0..1.0.
            sensor_val = random_uint() as f32 / RANDOM_UINT_MAX as f32;
        }

        Sensor::Signal0 => {
            // Returns magnitude of signal0 in the local neighborhood, with
            // 0.0..maxSignalSum converted to sensorRange 0.0..1.0
            sensor_val = get_signal_density(0, indiv.loc, signals, params);
        }

        Sensor::Signal0Fwd => {
            // Sense signal0 density along axis of last movement direction
            sensor_val = get_signal_density_along_axis(0, indiv.loc, indiv.last_move_dir, signals, params);
        }

        Sensor::Signal0Lr => {
            // Sense signal0 density along an axis perpendicular to last movement direction
            sensor_val = get_signal_density_along_axis(0, indiv.loc, indiv.last_move_dir.rotate90_deg_cw(), signals, params);
        }

        Sensor::GeneticSimFwd => {
            // Return minimum sensor value if nobody is alive in the forward adjacent location,
            // else returns a similarity match in the sensor range 0.0..1.0
            let loc2 = indiv.loc + indiv.last_move_dir;
            if grid.is_in_bounds(loc2) && grid.is_occupied_at(loc2) {
                if let Some(indiv2) = peeps.get_indiv(loc2, grid) {
                    if indiv2.alive {
                        sensor_val = genome_similarity_with_params(&indiv.genome, &indiv2.genome, params); // 0.0..1.0
                    }
                }
            }
        }

        Sensor::NumSenses => {
            sensor_val = 0.0;
        }
    }

    // Validate and clip sensor value
    if sensor_val.is_nan() || sensor_val < -0.01 || sensor_val > 1.01 {
        eprintln!("sensorVal={} for {:?}", sensor_val, sensor);
        sensor_val = sensor_val.max(0.0).min(1.0); // clip
    }

    assert!(!sensor_val.is_nan() && sensor_val >= -0.01 && sensor_val <= 1.01);

    sensor_val
}
