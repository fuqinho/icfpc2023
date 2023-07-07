use crate::RawAttendee;
use euclid::default::Point2D;
use lyon_geom::LineSegment;

use crate::problem::RawPlacement;
use crate::problem::RawProblem;
use crate::problem::RawSolution;

fn is_blocked(
    attendee: &RawAttendee,
    placement: &RawPlacement,
    placements: &[RawPlacement],
) -> bool {
    let p1 = Point2D::new(attendee.x, attendee.y);
    let p2 = Point2D::new(placement.x, placement.y);
    let segment = LineSegment { from: p1, to: p2 };
    for blocker in placements {
        if blocker == placement {
            continue;
        }
        if segment.square_distance_to_point(Point2D::new(blocker.x, blocker.y)) <= 25. {
            return true;
        }
    }
    false
}

fn evaluate_attendee(attendee: &RawAttendee, solution: &RawSolution) -> u64 {
    let mut score = 0u64;
    for (taste, placement) in attendee.tastes.iter().zip(solution.placements.iter()) {
        if is_blocked(attendee, placement, &solution.placements) {
            continue;
        }
        let p1 = Point2D::new(attendee.x, attendee.y);
        let p2 = Point2D::new(placement.x, placement.y);
        let d2 = LineSegment { from: p1, to: p2 }.square_length();
        score += (1000000000f64 * taste / d2).ceil() as u64;
    }
    score
}

pub fn evaluate(problem: &RawProblem, solution: &RawSolution) -> u64 {
    problem
        .attendees
        .iter()
        .map(|attendee| evaluate_attendee(attendee, solution))
        .sum()
}
