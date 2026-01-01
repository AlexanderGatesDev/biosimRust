// survival_criteria.rs
// Returns survival criterion results for individuals

use crate::indiv::Indiv;
use crate::grid::Grid;
use crate::params::Params;
use crate::grid::visit_neighborhood;
use crate::basic_types::Coord;
use crate::simulator::*;

// Returns (passed, score) where score is 0.0..1.0
pub fn passed_survival_criterion(
    indiv: &Indiv,
    challenge: u32,
    params: &Params,
    grid: &Grid,
) -> (bool, f32) {
    if !indiv.alive {
        return (false, 0.0);
    }

    match challenge {
        CHALLENGE_CIRCLE => {
            let safe_center = Coord::new(
                (params.size_x / 4) as i16,
                (params.size_y / 4) as i16,
            );
            let radius = params.size_x as f32 / 4.0;

            let offset = safe_center - indiv.loc;
            let distance = offset.length() as f32;
            if distance <= radius {
                (true, (radius - distance) / radius)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_RIGHT_HALF => {
            if indiv.loc.x > (params.size_x / 2) as i16 {
                (true, 1.0)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_RIGHT_QUARTER => {
            if indiv.loc.x > (params.size_x / 2 + params.size_x / 4) as i16 {
                (true, 1.0)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_LEFT_EIGHTH => {
            if indiv.loc.x < (params.size_x / 8) as i16 {
                (true, 1.0)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_STRING => {
            let min_neighbors = 2u32;
            let max_neighbors = 22u32;
            let radius = 1.5f32;

            if grid.is_border(indiv.loc) {
                return (false, 0.0);
            }

            let mut count = 0u32;
            visit_neighborhood(indiv.loc, radius, params, |loc2| {
                // Exclude the center location (the individual itself)
                if loc2 != indiv.loc && grid.is_occupied_at(loc2) {
                    count += 1;
                }
            });

            if count >= min_neighbors && count <= max_neighbors {
                (true, 1.0)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_CENTER_WEIGHTED => {
            let safe_center = Coord::new(
                (params.size_x / 2) as i16,
                (params.size_y / 2) as i16,
            );
            let radius = params.size_x as f32 / 3.0;

            let offset = safe_center - indiv.loc;
            let distance = offset.length() as f32;
            if distance <= radius {
                (true, (radius - distance) / radius)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_CENTER_UNWEIGHTED => {
            let safe_center = Coord::new(
                (params.size_x / 2) as i16,
                (params.size_y / 2) as i16,
            );
            let radius = params.size_x as f32 / 3.0;

            let offset = safe_center - indiv.loc;
            let distance = offset.length() as f32;
            if distance <= radius {
                (true, 1.0)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_CENTER_SPARSE => {
            let safe_center = Coord::new(
                (params.size_x / 2) as i16,
                (params.size_y / 2) as i16,
            );
            let outer_radius = params.size_x as f32 / 4.0;
            let inner_radius = 1.5f32;
            let min_neighbors = 5u32;
            let max_neighbors = 8u32;

            let offset = safe_center - indiv.loc;
            let distance = offset.length() as f32;
            if distance <= outer_radius {
                let mut count = 0u32;
                visit_neighborhood(indiv.loc, inner_radius, params, |loc2| {
                    if grid.is_occupied_at(loc2) {
                        count += 1;
                    }
                });
                if count >= min_neighbors && count <= max_neighbors {
                    return (true, 1.0);
                }
            }
            (false, 0.0)
        }

        CHALLENGE_CORNER => {
            assert_eq!(params.size_x, params.size_y);
            let radius = params.size_x as f32 / 8.0;

            let corners = [
                Coord::new(0, 0),
                Coord::new(0, (params.size_y - 1) as i16),
                Coord::new((params.size_x - 1) as i16, 0),
                Coord::new((params.size_x - 1) as i16, (params.size_y - 1) as i16),
            ];

            for corner in &corners {
                let distance = (*corner - indiv.loc).length() as f32;
                if distance <= radius {
                    return (true, 1.0);
                }
            }
            (false, 0.0)
        }

        CHALLENGE_CORNER_WEIGHTED => {
            assert_eq!(params.size_x, params.size_y);
            let radius = params.size_x as f32 / 4.0;

            let corners = [
                Coord::new(0, 0),
                Coord::new(0, (params.size_y - 1) as i16),
                Coord::new((params.size_x - 1) as i16, 0),
                Coord::new((params.size_x - 1) as i16, (params.size_y - 1) as i16),
            ];

            for corner in &corners {
                let distance = (*corner - indiv.loc).length() as f32;
                if distance <= radius {
                    return (true, (radius - distance) / radius);
                }
            }
            (false, 0.0)
        }

        CHALLENGE_RADIOACTIVE_WALLS => {
            (true, 1.0)
        }

        CHALLENGE_AGAINST_ANY_WALL => {
            let on_edge = indiv.loc.x == 0
                || indiv.loc.x == (params.size_x - 1) as i16
                || indiv.loc.y == 0
                || indiv.loc.y == (params.size_y - 1) as i16;

            if on_edge {
                (true, 1.0)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_TOUCH_ANY_WALL => {
            if indiv.challenge_bits != 0 {
                (true, 1.0)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_MIGRATE_DISTANCE => {
            let distance = (indiv.loc - indiv.birth_loc).length() as f32;
            let max_size = params.size_x.max(params.size_y) as f32;
            let required_distance = max_size / 2.0;
            if distance >= required_distance {
                (true, distance / max_size)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_EAST_WEST_EIGHTHS => {
            if indiv.loc.x < (params.size_x / 8) as i16
                || indiv.loc.x >= ((params.size_x - params.size_x / 8) as i16)
            {
                (true, 1.0)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_NEAR_BARRIER => {
            let radius = params.size_x as f32 / 2.0;
            let barrier_centers = grid.get_barrier_centers();
            let mut min_distance = 1e8f32;

            for center in barrier_centers {
                let distance = (indiv.loc - *center).length() as f32;
                if distance < min_distance {
                    min_distance = distance;
                }
            }

            if min_distance <= radius {
                (true, 1.0 - (min_distance / radius))
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_PAIRS => {
            let on_edge = indiv.loc.x == 0
                || indiv.loc.x == (params.size_x - 1) as i16
                || indiv.loc.y == 0
                || indiv.loc.y == (params.size_y - 1) as i16;

            if on_edge {
                return (false, 0.0);
            }

            let mut count = 0u32;
            for x in (indiv.loc.x - 1)..=(indiv.loc.x + 1) {
                for y in (indiv.loc.y - 1)..=(indiv.loc.y + 1) {
                    let tloc = Coord::new(x, y);
                    if tloc != indiv.loc && grid.is_in_bounds(tloc) && grid.is_occupied_at(tloc) {
                        count += 1;
                        if count == 1 {
                            for x1 in (tloc.x - 1)..=(tloc.x + 1) {
                                for y1 in (tloc.y - 1)..=(tloc.y + 1) {
                                    let tloc1 = Coord::new(x1, y1);
                                    if tloc1 != tloc
                                        && tloc1 != indiv.loc
                                        && grid.is_in_bounds(tloc1)
                                        && grid.is_occupied_at(tloc1)
                                    {
                                        return (false, 0.0);
                                    }
                                }
                            }
                        } else {
                            return (false, 0.0);
                        }
                    }
                }
            }
            if count == 1 {
                (true, 1.0)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_LOCATION_SEQUENCE => {
            let mut count = 0u32;
            let bits = indiv.challenge_bits;
            let max_number_of_bits = 32u32;

            for n in 0..max_number_of_bits {
                if (bits & (1u32 << n)) != 0 {
                    count += 1;
                }
            }
            if count > 0 {
                (true, count as f32 / max_number_of_bits as f32)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_ALTRUISM_SACRIFICE => {
            let radius = params.size_x as f32 / 4.0;
            let target = Coord::new(
                (params.size_x - params.size_x / 4) as i16,
                (params.size_y - params.size_y / 4) as i16,
            );
            let distance = (target - indiv.loc).length() as f32;
            if distance <= radius {
                (true, (radius - distance) / radius)
            } else {
                (false, 0.0)
            }
        }

        CHALLENGE_ALTRUISM => {
            let safe_center = Coord::new(
                (params.size_x / 4) as i16,
                (params.size_y / 4) as i16,
            );
            let radius = params.size_x as f32 / 4.0;

            let offset = safe_center - indiv.loc;
            let distance = offset.length() as f32;
            if distance <= radius {
                (true, (radius - distance) / radius)
            } else {
                (false, 0.0)
            }
        }

        _ => {
            panic!("Unknown challenge: {}", challenge);
        }
    }
}

