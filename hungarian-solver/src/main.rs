mod solver;

use anyhow::Result;
use common::{api::Client, evaluate, Solution};

use crate::solver::Solver;

#[argopt::cmd]
fn main(problem_id: u32) -> Result<()> {
    let cl = Client::new();
    let userboard = cl.get_userboard()?;

    let best_score = userboard.problems[(problem_id - 1) as usize].unwrap_or(0.);

    eprintln!("our best score: {}", best_score);

    let problem = cl.get_problem(problem_id)?;

    let mut solver = Solver::new(problem_id, problem.clone());

    let (score, board) = solver.solve();

    eprintln!("final score: {}", score);

    let solution: Solution = board.try_into().unwrap();

    let eval_score = evaluate(&problem, &solution);

    // assert_eq!(score, eval_score);

    if eval_score > best_score {
        cl.post_submission(problem_id, solution)?;

        eprintln!("Submitted solution for problem {}!", problem_id);
    }

    Ok(())
}
