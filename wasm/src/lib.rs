mod annealing;

pub use annealing::*;

use wasm_bindgen::{prelude::wasm_bindgen, JsError, JsValue};

/// Wraps [`anyhow::Error`] for conversion to [`JsValue`].
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct Error(#[from] anyhow::Error);

impl From<Error> for JsValue {
    fn from(value: Error) -> JsValue {
        JsValue::from(JsError::from(value))
    }
}

/// Specialized [`std::result::Result`] for [`Error`].
type Result<T> = std::result::Result<T, Error>;

#[wasm_bindgen]
pub struct ProblemHandle {
    problem: common::Problem,
}

#[wasm_bindgen]
impl ProblemHandle {
    pub fn from_json(json: &str) -> Result<ProblemHandle> {
        let problem = common::RawProblem::from_json(json)?;
        Ok(ProblemHandle {
            problem: problem.into(),
        })
    }
}

#[wasm_bindgen]
pub struct SolutionHandle {
    solution: common::Solution,
}

#[wasm_bindgen]
impl SolutionHandle {
    pub fn from_json(json: &str) -> Result<SolutionHandle> {
        let solution = common::RawSolution::from_json(json)?;
        Ok(SolutionHandle {
            solution: solution.into(),
        })
    }
}

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
