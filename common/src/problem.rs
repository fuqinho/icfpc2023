use serde::{Deserialize, Serialize};

use euclid::default::{Box2D, Point2D};
use std::convert::From;

#[derive(Clone, Debug)]
pub struct Problem {
    pub room: Box2D<f64>,
    pub stage: Box2D<f64>,
    pub musicians: Vec<usize>,
    pub attendees: Vec<Attendee>,
}

#[derive(Clone, Debug)]
pub struct Attendee {
    pub position: Point2D<f64>,
    pub tastes: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct Solution {
    pub placements: Vec<Placement>,
}

#[derive(Clone, Debug)]
pub struct Placement {
    pub position: Point2D<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct RawProblem {
    pub room_width: f64,
    pub room_height: f64,
    pub stage_width: f64,
    pub stage_height: f64,
    // x, y
    pub stage_bottom_left: Vec<f64>,
    pub musicians: Vec<usize>,
    pub attendees: Vec<RawAttendee>,
}

impl RawProblem {
    pub fn from_json(s: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&s)?)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct RawAttendee {
    pub x: f64,
    pub y: f64,
    pub tastes: Vec<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct RawSolution {
    pub placements: Vec<RawPlacement>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct RawPlacement {
    pub x: f64,
    pub y: f64,
}

impl From<RawAttendee> for Attendee {
    fn from(raw: RawAttendee) -> Self {
        Self {
            position: Point2D::new(raw.x, raw.y),
            tastes: raw.tastes,
        }
    }
}

impl From<RawProblem> for Problem {
    fn from(raw: RawProblem) -> Self {
        Self {
            room: Box2D::new(
                Point2D::new(0.0, 0.0),
                Point2D::new(raw.room_width, raw.room_height),
            ),
            stage: Box2D::new(
                Point2D::new(raw.stage_bottom_left[0], raw.stage_bottom_left[1]),
                Point2D::new(
                    raw.stage_bottom_left[0] + raw.stage_width,
                    raw.stage_bottom_left[1] + raw.stage_width,
                ),
            ),
            musicians: raw.musicians,
            attendees: raw
                .attendees
                .into_iter()
                .map(Attendee::from)
                .collect::<Vec<_>>(),
        }
    }
}

impl From<Solution> for RawSolution {
    fn from(s: Solution) -> Self {
        Self {
            placements: s
                .placements
                .into_iter()
                .map(RawPlacement::from)
                .collect::<Vec<_>>(),
        }
    }
}

impl From<RawSolution> for Solution {
    fn from(raw: RawSolution) -> Self {
        Self {
            placements: raw
                .placements
                .into_iter()
                .map(Placement::from)
                .collect::<Vec<_>>(),
        }
    }
}

impl From<Placement> for RawPlacement {
    fn from(p: Placement) -> Self {
        Self {
            x: p.position.x,
            y: p.position.y,
        }
    }
}

impl From<RawPlacement> for Placement {
    fn from(raw: RawPlacement) -> Self {
        Self {
            position: Point2D::new(raw.x, raw.y),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::problem::RawPlacement;
    use crate::problem::RawProblem;
    use crate::problem::RawSolution;

    #[test]
    fn deserialize_test() {
        let example = r#"{
            "room_width": 2000.0,
            "room_height": 5000.0,
            "stage_width": 1000.0,
            "stage_height": 200.0,
            "stage_bottom_left": [500.0, 0.0],
            "musicians": [0, 1, 0],
            "attendees": [
            { "x": 100.0, "y": 500.0, "tastes": [1000.0, -1000.0
            ] },
            { "x": 200.0, "y": 1000.0, "tastes": [200.0, 200.0]
            },
            { "x": 1100.0, "y": 800.0, "tastes": [800.0, 1500.0]
            }
            ]
            }"#;

        let p: RawProblem = serde_json::from_str(&example).unwrap();

        assert_eq!(p.room_width, 2000.0);
        assert_eq!(p.stage_bottom_left, vec![500.0, 0.0]);
        assert_eq!(p.attendees[0].tastes, vec![1000.0, -1000.0]);
    }

    #[test]
    fn serialize_test() {
        let solution = RawSolution {
            placements: vec![
                RawPlacement { x: 100.0, y: 200.0 },
                RawPlacement { x: 300.5, y: 400.5 },
            ],
        };

        let s = serde_json::to_string(&solution).expect("failed to serialize");
        assert_eq!(
            s,
            r#"{"placements":[{"x":100.0,"y":200.0},{"x":300.5,"y":400.5}]}"#
        );
    }
}
