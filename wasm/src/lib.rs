mod annealing;

pub use annealing::*;

use wasm_bindgen::{prelude::*, JsError, JsValue};

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Wraps various error types for conversion to [`JsValue`].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

impl From<Error> for JsValue {
    fn from(value: Error) -> JsValue {
        JsValue::from(JsError::from(value))
    }
}

/// Specialized [`std::result::Result`] for [`Error`].
type Result<T> = std::result::Result<T, Error>;

#[wasm_bindgen]
pub struct ProblemHandle {
    pub(crate) real: common::Problem,
}

#[wasm_bindgen]
impl ProblemHandle {
    pub fn from_json(json: &str) -> Result<ProblemHandle> {
        let problem = common::RawProblem::from_json(json)?;
        Ok(ProblemHandle {
            real: problem.into(),
        })
    }
}

impl From<common::Problem> for ProblemHandle {
    fn from(problem: common::Problem) -> Self {
        Self { real: problem }
    }
}

#[wasm_bindgen]
pub struct SolutionHandle {
    pub(crate) real: common::Solution,
}

#[wasm_bindgen]
impl SolutionHandle {
    pub fn from_json(json: &str) -> Result<SolutionHandle> {
        let solution = common::RawSolution::from_json(json)?;
        Ok(SolutionHandle {
            real: solution.into(),
        })
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&common::RawSolution::from(
            self.real.clone(),
        ))?)
    }
}

impl From<common::Solution> for SolutionHandle {
    fn from(solution: common::Solution) -> Self {
        Self { real: solution }
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
