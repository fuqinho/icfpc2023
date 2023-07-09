use common::{Problem, Solution};

const SOLVER_NAME: &str = "chain-solver";

pub fn solve(_problem: &Problem) -> Solution {
    Solution {
        problem_id: 0,
        solver: SOLVER_NAME.to_owned(),
        placements: vec![],
    }
}
