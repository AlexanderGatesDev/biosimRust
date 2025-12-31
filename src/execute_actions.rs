// execute_actions.rs - Executes the actions computed for a single simStep

use crate::indiv::Indiv;
use crate::sensors_actions::Action;
use crate::sensors_actions::{NUM_ACTIONS, ACTION_MIN, ACTION_RANGE};
use crate::random::{random_uint, RANDOM_UINT_MAX};
use crate::params::Params;
use crate::peeps::Peeps;
use crate::grid::Grid;
use crate::signals::Signals;
use crate::basic_types::Coord;

pub fn prob2bool(factor: f32) -> bool {
    assert!(factor >= 0.0 && factor <= 1.0);
    (random_uint() as f32 / RANDOM_UINT_MAX as f32) < factor
}

pub fn response_curve(r: f32, k: u32) -> f32 {
    let kf = k as f32;
    (r - 2.0).powf(-2.0 * kf) - (2.0f32).powf(-2.0 * kf) * (1.0 - r)
}

fn is_enabled(action: Action, action_levels: &[f32; NUM_ACTIONS]) -> bool {
    let idx = action.as_usize();
    if idx >= NUM_ACTIONS {
        return false;
    }
    let level = action_levels[idx];
    let level01 = (level.tanh() + 1.0) / 2.0;
    level01 > ACTION_MIN && prob2bool((level01 - ACTION_MIN) / ACTION_RANGE)
}

pub fn execute_actions(
    indiv: &mut Indiv,
    action_levels: &[f32; NUM_ACTIONS],
    params: &Params,
    peeps: &Peeps,
    grid: &Grid,
    signals: &mut Signals,
) {
    // Responsiveness action
    if (Action::SetResponsiveness as usize) < NUM_ACTIONS {
        let level = action_levels[Action::SetResponsiveness as usize];
        let level = (level.tanh() + 1.0) / 2.0;
        indiv.responsiveness = level;
    }
    
    let responsiveness_adjusted = response_curve(indiv.responsiveness, params.responsiveness_curve_k_factor);
    
    // Oscillator period action
    if (Action::SetOscillatorPeriod as usize) < NUM_ACTIONS {
        let periodf = action_levels[Action::SetOscillatorPeriod as usize];
        let new_periodf01 = (periodf.tanh() + 1.0) / 2.0;
        let new_period = 1 + (1.5 + (7.0 * new_periodf01).exp()) as u32;
        indiv.osc_period = new_period.max(2).min(2048);
    }
    
    // Set longProbeDistance
    if (Action::SetLongprobeDist as usize) < NUM_ACTIONS {
        const MAX_LONG_PROBE_DISTANCE: u32 = 32;
        let level = action_levels[Action::SetLongprobeDist as usize];
        let level01 = (level.tanh() + 1.0) / 2.0;
        let new_dist = 1 + (level01 * MAX_LONG_PROBE_DISTANCE as f32) as u32;
        indiv.long_probe_dist = new_dist;
    }
    
    // Kill forward action
    if params.kill_enable {
        let kill_idx = Action::KillForward as usize;
        if kill_idx < NUM_ACTIONS {
            const KILL_THRESHOLD: f32 = 0.5;
            let mut level = action_levels[kill_idx];
            level = (level.tanh() + 1.0) / 2.0;
            level *= responsiveness_adjusted;
            if level > KILL_THRESHOLD && prob2bool((level - ACTION_MIN) / ACTION_RANGE) {
                let other_loc = indiv.loc + indiv.last_move_dir.as_normalized_coord();
                if grid.is_in_bounds(other_loc) && grid.is_occupied_at(other_loc) {
                    if let Some(target_indiv) = peeps.get_indiv(other_loc, grid) {
                        let distance = (indiv.loc - target_indiv.loc).length();
                        if distance == 1 {
                            peeps.queue_for_death(target_indiv);
                        }
                    }
                }
            }
        }
    }
    
    // Emit signal0
    if is_enabled(Action::EmitSignal0, action_levels) {
        const EMIT_THRESHOLD: f32 = 0.5;
        let mut level = action_levels[Action::EmitSignal0 as usize];
        level = (level.tanh() + 1.0) / 2.0;
        level *= responsiveness_adjusted;
        if level > EMIT_THRESHOLD && prob2bool(level) {
            signals.increment(0, indiv.loc, params);
        }
    }
    
    // Movement action neurons
    let last_move_offset = indiv.last_move_dir.as_normalized_coord();
    let mut move_x = action_levels[Action::MoveX as usize];
    let mut move_y = action_levels[Action::MoveY as usize];
    move_x += action_levels[Action::MoveEast as usize];
    move_x -= action_levels[Action::MoveWest as usize];
    move_y += action_levels[Action::MoveNorth as usize];
    move_y -= action_levels[Action::MoveSouth as usize];
    
    let level_forward = action_levels[Action::MoveForward as usize];
    move_x += last_move_offset.x as f32 * level_forward;
    move_y += last_move_offset.y as f32 * level_forward;
    
    let level_reverse = action_levels[Action::MoveReverse as usize];
    move_x -= last_move_offset.x as f32 * level_reverse;
    move_y -= last_move_offset.y as f32 * level_reverse;
    
    let level_left = action_levels[Action::MoveLeft as usize];
    let offset_left = indiv.last_move_dir.rotate90_deg_ccw().as_normalized_coord();
    move_x += offset_left.x as f32 * level_left;
    move_y += offset_left.y as f32 * level_left;
    
    let level_right = action_levels[Action::MoveRight as usize];
    let offset_right = indiv.last_move_dir.rotate90_deg_cw().as_normalized_coord();
    move_x += offset_right.x as f32 * level_right;
    move_y += offset_right.y as f32 * level_right;
    
    let level_rl = action_levels[Action::MoveRl as usize];
    let offset_rl = indiv.last_move_dir.rotate90_deg_cw().as_normalized_coord();
    move_x += offset_rl.x as f32 * level_rl;
    move_y += offset_rl.y as f32 * level_rl;
    
    let level_random = action_levels[Action::MoveRandom as usize];
    use crate::basic_types::Dir;
    let offset_random = Dir::random8().as_normalized_coord();
    move_x += offset_random.x as f32 * level_random;
    move_y += offset_random.y as f32 * level_random;
    
    move_x = move_x.tanh();
    move_y = move_y.tanh();
    move_x *= responsiveness_adjusted;
    move_y *= responsiveness_adjusted;
    
    let prob_x = if prob2bool(move_x.abs()) { 1i16 } else { 0i16 };
    let prob_y = if prob2bool(move_y.abs()) { 1i16 } else { 0i16 };
    
    let signum_x = if move_x < 0.0 { -1i16 } else { 1i16 };
    let signum_y = if move_y < 0.0 { -1i16 } else { 1i16 };
    let movement_offset = Coord {
        x: prob_x * signum_x,
        y: prob_y * signum_y,
    };
    
    let new_loc = indiv.loc + movement_offset;
    if grid.is_in_bounds(new_loc) && grid.is_empty_at(new_loc) {
        if movement_offset.x != 0 || movement_offset.y != 0 {
            peeps.queue_for_move(indiv, new_loc);
        }
    }
}
