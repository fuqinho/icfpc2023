use common::evaluate;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct RawProblem(common::RawProblem);

#[wasm_bindgen]
impl RawProblem {
    pub fn from_json(s: &str) -> Self {
        Self(common::RawProblem::from_json(s).unwrap())
    }

    pub fn room_width(&self) -> f64 {
        self.0.room_width
    }

    pub fn room_height(&self) -> f64 {
        self.0.room_height
    }

    pub fn stage_width(&self) -> f64 {
        self.0.stage_width
    }

    pub fn stage_height(&self) -> f64 {
        self.0.stage_height
    }

    pub fn stage_bottom_left(&self) -> Vec<f64> {
        self.0.stage_bottom_left.to_vec()
    }

    pub fn musicians(&self) -> Vec<usize> {
        self.0.musicians.to_vec()
    }

    pub fn attendees_len(&self) -> usize {
        self.0.attendees.len()
    }

    pub fn attendee_x(&self, i: usize) -> f64 {
        self.0.attendees[i].x
    }

    pub fn attendee_y(&self, i: usize) -> f64 {
        self.0.attendees[i].y
    }

    pub fn attendee_tastes(&self, i: usize) -> Vec<f64> {
        self.0.attendees[i].tastes.to_vec()
    }
}

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct EvaluationResult {
    pub score: f64,
}

#[wasm_bindgen]
impl EvaluationResult {
    pub fn from_json(problem: &str, solution: &str) -> Self {
        let problem = common::Problem::from(common::RawProblem::from_json(problem).unwrap());
        let solution = common::Solution::from(common::RawSolution::from_json(solution).unwrap());
        return Self {
            score: evaluate(&problem, &solution),
        };
    }

    pub fn to_json(&self) -> String {
        return serde_json::to_string(&self).unwrap();
    }
}
