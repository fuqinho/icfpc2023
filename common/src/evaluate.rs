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

fn evaluate_attendee(attendee: &Attendee, musicians: &[usize], solution: &Solution) -> u64 {
    let mut score = 0u64;
    for (inst, placement) in musicians.iter().zip(solution.placements.iter()) {
        if is_blocked(attendee, placement, &solution.placements) {
            continue;
        }
        let d2 = LineSegment {
            from: attendee.position,
            to: placement.position,
        }
        .square_length();
        score += (1000000f64 * attendee.tastes[*inst] / d2).ceil() as u64;
    }
    score
}

pub fn evaluate(problem: &Problem, solution: &Solution) -> u64 {
    problem
        .attendees
        .iter()
        .map(|attendee| evaluate_attendee(attendee, &problem.musicians, solution))
        .sum()
}
