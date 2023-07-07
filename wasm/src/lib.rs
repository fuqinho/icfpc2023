use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct RawProblem(common::RawProblem);

#[wasm_bindgen]
impl RawProblem {
    pub fn from_json(s: &str) -> Self {
        Self(common::RawProblem::from_json(s).unwrap())
    }
}
