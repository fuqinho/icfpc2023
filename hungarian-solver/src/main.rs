mod solver;

use anyhow::Result;
use common::{
    api::{get_best_solution, Client},
    evaluate, Solution,
};

use crate::solver::Solver;

#[argopt::cmd]
fn main(
    problem_id: u32,
    #[opt(short, long, default_value = "normal")] algo: solver::Algorithm,
    #[opt(short, long, default_value = "")] out: String,
    #[opt(short, long)] submit_must: bool,
    #[opt(long)] no_post_process: bool,
) -> Result<()> {
    let cl = Client::new();

    let problem = cl.get_problem(problem_id)?;

    let best_solution = get_best_solution(problem_id).unwrap();

    let best_score = evaluate(&problem, &best_solution);

    eprintln!("our best score: {}", best_score);

    let mut solver = Solver::new(problem_id, problem.clone(), algo);

    let (_score, board) = solver.solve(!no_post_process);

    let solution: Solution = board.solution_with_optimized_volume().unwrap();

    let eval_score = evaluate(&problem, &solution);

    eprintln!("final score: {}", eval_score);

    let improve_percent = eval_score as f64 / best_score * 100.0 - 100.0;

    if improve_percent > 0. || submit_must {
        if improve_percent > 0. {
            eprintln!("score improved by {:.2}%", improve_percent);
        }

        cl.post_submission(problem_id, solution.clone())?;

        eprintln!("Submitted solution for problem {}!", problem_id);
    }

    if out != "" {
        eprintln!("Writing solution to {}", out);
        Solution::write_to_file(out, solution)?;
    }

    Ok(())
}
