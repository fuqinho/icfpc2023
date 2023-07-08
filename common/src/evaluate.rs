use euclid::default::{Box2D, Point2D, Vector2D};
use lyon_geom::LineSegment;

use crate::problem::Attendee;
use crate::problem::Placement;
use crate::problem::Problem;
use crate::problem::Solution;

pub fn is_blocked(attendee: &Attendee, placement: &Placement, placements: &[Placement]) -> bool {
    let segment = LineSegment {
        from: attendee.position,
        to: placement.position,
    };
    for blocker in placements {
        if blocker.position == placement.position {
            continue;
        }
        if segment.square_distance_to_point(blocker.position) <= 25. {
            return true;
        }
    }
    false
}

fn is_blocked_internal(
    seg: &LineSegment<f64>,
    current_index: usize,
    placements: &[Placement],
) -> bool {
    for (i, blocker) in placements.iter().enumerate() {
        if i == current_index {
            continue;
        }
        if seg.distance_to_point(blocker.position) <= 5. {
            return true;
        }
    }
    false
}

fn evaluate_attendee(attendee: &Attendee, musicians: &[usize], solution: &Solution) -> f64 {
    let mut score = 0f64;
    for (index, (inst, placement)) in musicians.iter().zip(solution.placements.iter()).enumerate() {
        let seg = LineSegment {
            from: attendee.position,
            to: placement.position,
        };
        if is_blocked_internal(&seg, index, &solution.placements) {
            continue;
        }
        let d = seg.length();
        score += (1000000f64 * attendee.tastes[*inst] / (d * d)).ceil();
    }
    score
}

pub fn evaluate(problem: &Problem, solution: &Solution) -> f64 {
    problem
        .attendees
        .iter()
        .map(|attendee| evaluate_attendee(attendee, &problem.musicians, solution))
        .sum()
}

// Find rough upper bound.
pub fn estimate(problem_id: u32, problem: &Problem) -> (f64, Solution) {
    let p1 = Point2D::new(problem.stage.min.x + 10., problem.stage.min.y + 10.);
    let p2 = Point2D::new(problem.stage.min.x + 10., problem.stage.max.y - 10.);
    let p3 = Point2D::new(problem.stage.max.x - 10., problem.stage.max.y - 10.);
    let p4 = Point2D::new(problem.stage.max.x - 10., problem.stage.min.y + 10.);

    let s1 = LineSegment { from: p1, to: p2 };
    let s2 = LineSegment { from: p2, to: p3 };
    let s3 = LineSegment { from: p3, to: p4 };
    let s4 = LineSegment { from: p4, to: p1 };

    let mut candidates = vec![];
    for a in problem.attendees.iter() {
        let mut min_dist_sq = f64::MAX;
        let mut candidate = Point2D::new(0., 0.);
        for s in [&s1, &s2, &s3, &s4] {
            let p = s.closest_point(a.position);
            let dist_sq = (p - a.position).square_length();
            if min_dist_sq > dist_sq {
                min_dist_sq = dist_sq;
                candidate = p;
            }
        }
        candidates.push(candidate);
    }
    candidates.push(problem.stage.center());

    let mut placements = vec![];
    for (idx, inst) in problem.musicians.iter().enumerate() {
        let mut pos = Point2D::new(0., 0.);
        let mut max_score = f64::MIN;
        for candidate in candidates.iter() {
            let mut score = 0.;
            for a in problem.attendees.iter() {
                score += (1_000_000f64 * a.tastes[*inst]
                    / LineSegment {
                        from: a.position,
                        to: *candidate,
                    }
                    .square_length())
                .ceil();
            }
            if max_score < score {
                max_score = score;
                pos = *candidate;
            }
        }

        if placements.iter().any(|p: &Placement| {
            LineSegment {
                from: p.position,
                to: pos,
            }
            .square_length()
                < 100.
        }) {
            // conflict.
            let box2d = Box2D {
                min: problem.stage.min + Vector2D::new(10., 10.),
                max: problem.stage.max - Vector2D::new(10., 10.),
            };
            let mut found = false;
            for i in (1..problem.musicians.len() as isize) {
                let mut max_score = f64::MIN;
                let mut max_pos = Point2D::new(0., 0.);
                for ix in -i..=i {
                    for iy in -i..=i {
                        let candidate = pos + Vector2D::new((10 * ix) as f64, (10 * iy) as f64);
                        if candidate.x < box2d.min.x
                            || box2d.max.x < candidate.x
                            || candidate.y < box2d.min.y
                            || box2d.min.y < candidate.y
                        {
                            continue;
                        }
                        if placements.iter().any(|p: &Placement| {
                            LineSegment {
                                from: p.position,
                                to: candidate,
                            }
                            .square_length()
                                < 100.
                        }) {
                            continue;
                        }
                        let mut score = 0.;
                        for a in problem.attendees.iter() {
                            score += (1_000_000f64 * a.tastes[*inst]
                                / LineSegment {
                                    from: a.position,
                                    to: candidate,
                                }
                                .square_length())
                            .ceil();
                        }
                        if max_score < score {
                            max_score = score;
                            max_pos = candidate;
                            found = true;
                        }
                    }
                }
                if found {
                    pos = max_pos;
                    break;
                }
            }
            if !found {
                panic!("cannot avoided");
            }
        }
        placements.push(Placement { position: pos });
    }
    let solution = Solution {
        problem_id,
        placements,
    };
    let score = evaluate(problem, &solution);
    (score, solution)
}
