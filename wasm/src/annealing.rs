use tanakh_solver::solver::Solver2;
use wasm_bindgen::prelude::*;

use crate::{ProblemHandle, SolutionHandle};

#[wasm_bindgen]
pub fn perform_annealing(
    problem: &ProblemHandle,
    initial_solution: &SolutionHandle,
    temp: f64,
    time_limit: f64,
    seed: u64,
) -> SolutionHandle {
    let solver = Solver2 {
        problem_id: initial_solution.real.problem_id,
        problem: &problem.real,
        start_temp: Some(temp),
        better_initial: false,
        initial_solution: Some(&initial_solution.real),
        taste: None,
        param: String::new(),
    };

    let result = saru::annealing(
        &solver,
        &saru::AnnealingOptions {
            time_limit,
            limit_temp: temp,
            restart: 0,
            threads: 1,
            silent: false,
            header: String::new(),
        },
        seed,
    );

    let mut solution = result.solution.expect("Valid solution not found");
    solution.solver = initial_solution.real.solver.clone();
    if !solution.solver.ends_with("+anneal") {
        solution.solver = format!("{}+anneal", solution.solver);
    }

    solution.into()
}
