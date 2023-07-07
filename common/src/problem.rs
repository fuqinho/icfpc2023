use serde::{Deserialize, Serialize};

use crate::Point;

pub type P = Point<f64>;

// pub struct Problem {
//     room_width: f64,
//     room_height: f64,
//     stage_width: f64,
//     stage_height: f64,
//     stage_bottom_left: Point<f64>,
//     musicians: Vec<usize>,
//     attendees: Vec<
// }

// pub struct Attendee {
//     position: P,
//     tastes: Vec<f64>,
// }

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


#[cfg(test)]
mod tests {
    use crate::problem::RawProblem;
    use crate::problem::RawSolution;
    use crate::problem::RawPlacement;

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
            placements: vec![RawPlacement{x: 100.0, y: 200.0}, RawPlacement{x: 300.5, y: 400.5}],
        };

        let s = serde_json::to_string(&solution).expect("failed to serialize");
        assert_eq!(s, r#"{"placements":[{"x":100.0,"y":200.0},{"x":300.5,"y":400.5}]}"#);
    }
}
