use saru::{annealing_single_thread, State};
use tanakh_solver::solver::{Solver2, State2};
use wasm_bindgen::prelude::*;

use crate::{ProblemHandle, SolutionHandle};

#[wasm_bindgen]
pub struct SolverHandle {
    solver: Solver2,
    state: Option<State2>,
}

#[wasm_bindgen]
impl SolverHandle {
    pub fn new(problem: &ProblemHandle, initial_solution: &SolutionHandle) -> Self {
        let solver = Solver2 {
            problem_id: initial_solution.real.problem_id,
            problem: problem.real.clone(),
            start_temp: None,
            better_initial: false,
            initial_solution: None,
            taste: None,
            param: String::new(),
        };
        let mut solver_name = initial_solution.real.solver.clone();
        if !solver_name.ends_with("+anneal") {
            solver_name = format!("{}+anneal", solver_name);
        };
        let state = State2::new(&initial_solution.real, &problem.real, &solver_name);
        Self {
            solver,
            state: Some(state),
        }
    }

    pub fn run(&mut self, temp: f64, time_limit: f64, seed: u64) {
        self.solver.start_temp = Some(temp);
        let opts = saru::AnnealingOptions {
            time_limit,
            limit_temp: temp,
            restart: 0,
            silent: false,
            header: String::new(),
        };
        let result =
            annealing_single_thread(None, &self.solver, &opts, seed, self.state.take().unwrap());
        self.state = Some(result.state);
    }

    pub fn solution(&self) -> SolutionHandle {
        SolutionHandle::from(self.state.as_ref().unwrap().solution())
    }
}
