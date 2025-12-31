// end_of_sim_step.rs
// At the end of each sim step, handle deaths, movements, signal fading, and video frames

use crate::params::Params;
use crate::peeps::Peeps;
use crate::signals::Signals;
use crate::grid::Grid;
use crate::image_writer::ImageWriter;
use crate::random::{random_uint, RANDOM_UINT_MAX};
use crate::simulator::CHALLENGE_RADIOACTIVE_WALLS;
use crate::simulator::CHALLENGE_TOUCH_ANY_WALL;
use crate::simulator::CHALLENGE_LOCATION_SEQUENCE;

pub fn end_of_sim_step(
    sim_step: u32,
    generation: u32,
    params: &Params,
    peeps: &mut Peeps,
    grid: &mut Grid,
    signals: &mut Signals,
    image_writer: &mut ImageWriter,
) {
    // Radioactive walls challenge
    if params.challenge == CHALLENGE_RADIOACTIVE_WALLS {
        let radioactive_x = if sim_step < params.steps_per_generation / 2 {
            0i16
        } else {
            (params.size_x - 1) as i16
        };

        for index in 1..=params.population {
            if let Some(indiv) = peeps.get(index as u16) {
                if indiv.alive {
                    let distance_from_radioactive_wall =
                        (indiv.loc.x - radioactive_x).abs() as u32;
                    if distance_from_radioactive_wall < params.size_x as u32 / 2 {
                        let chance_of_death = 1.0 / distance_from_radioactive_wall as f32;
                        if (random_uint() as f32 / RANDOM_UINT_MAX as f32) < chance_of_death {
                            peeps.queue_for_death(indiv);
                        }
                    }
                }
            }
        }
    }

    // Touch any wall challenge
    if params.challenge == CHALLENGE_TOUCH_ANY_WALL {
        for index in 1..=params.population {
            if let Some(indiv) = peeps.get_mut(index as u16) {
                if indiv.loc.x == 0
                    || indiv.loc.x == (params.size_x - 1) as i16
                    || indiv.loc.y == 0
                    || indiv.loc.y == (params.size_y - 1) as i16
                {
                    indiv.challenge_bits = 1;
                }
            }
        }
    }

    // Location sequence challenge
    if params.challenge == CHALLENGE_LOCATION_SEQUENCE {
        let radius = 9.0f32;
        for index in 1..=params.population {
            if let Some(indiv) = peeps.get_mut(index as u16) {
                for (n, center) in grid.get_barrier_centers().iter().enumerate() {
                    let bit = 1u32 << n;
                    if (indiv.challenge_bits & bit) == 0 {
                        if (indiv.loc - *center).length() as f32 <= radius {
                            indiv.challenge_bits |= bit;
                        }
                        break;
                    }
                }
            }
        }
    }

    peeps.drain_death_queue(grid);
    peeps.drain_move_queue(grid);
    signals.fade(0, params); // Fade layer 0

    // Save video frame (or display frame if display is enabled)
    let should_save_frame = params.save_video
        && ((generation % params.video_stride == 0)
            || generation <= params.video_save_first_frames
            || (generation >= params.parameter_change_generation_number
                && generation
                    <= params.parameter_change_generation_number
                        + params.video_save_first_frames));
    
    // Always generate frames if display is enabled, regardless of save_video
    if should_save_frame || params.display_enabled {
        if !image_writer.save_video_frame_sync(sim_step, generation, grid, peeps, params) {
            // imageWriter busy
        }
    }
}

