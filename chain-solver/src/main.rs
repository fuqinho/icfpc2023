mod solver;

use anyhow::Result;
use common::{api::Client, evaluate, Solution};

use crate::solver::Solver;

#[argopt::cmd]
fn main(
    problem_id: u32,
    #[opt(short, long, default_value = "10")] tl_secs: u64,
    #[opt(short, long, default_value = "")] out: String,
    #[opt(short, long, default_value = "false")] submit_must: bool,
) -> Result<()> {
    let cl = Client::new();
    let userboard = cl.get_userboard()?;

    let best_score = userboard.problems[(problem_id - 1) as usize].unwrap_or(0.);

    eprintln!("our best score: {}", best_score);

    let problem = cl.get_problem(problem_id)?;

    let mut solver = Solver::new(problem_id, problem.clone(), 42);

    let (_score, board) = solver.solve(tl_secs);

    for i in 0..board.prob.musicians.len() {
        let cont = board.contribution(i);
        if cont < 0. {
            eprintln!("musician {i} is negatively contributing: {}", cont);
        }
    }

    let solution: Solution = board.try_into().unwrap();

    let eval_score = evaluate(&problem, &solution);

    eprintln!("final score: {}", eval_score);

    // assert_eq!(score, eval_score);

    if eval_score > best_score || submit_must {
        cl.post_submission(problem_id, solution.clone())?;

        eprintln!("Submitted solution for problem {}!", problem_id);
    }

    if out != "" {
        eprintln!("Writing solution to {}", out);
        Solution::write_to_file(out, solution)?;
    }

    Ok(())
}
