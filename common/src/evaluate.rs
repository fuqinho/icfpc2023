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
