mod annealing;

pub use annealing::*;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct Evaluator {}

#[wasm_bindgen]
impl Evaluator {
    pub fn from_json(
        problem: &str,
        solution: &str,
        detailed_item: &str,
        detailed_index: usize,
    ) -> String {
        // https://github.com/rustwasm/wasm-bindgen/issues/111
        // Vec<T>が返せないのでStringにして返しています。悲しい。
        let problem = common::Problem::from(common::RawProblem::from_json(problem).unwrap());
        let solution = common::Solution::from(common::RawSolution::from_json(solution).unwrap());
        return common::EvaluationResult::evaluate(
            &problem,
            &solution,
            detailed_item,
            detailed_index,
        )
        .to_json();
    }
}
