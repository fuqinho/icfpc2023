mod solver;

use anyhow::Result;
use common::{api::Client, board::Board, evaluate, Problem, Solution};
use lyon_geom::Point;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::solver::Solver;

#[argopt::cmd]
fn main(
    /// time limit in seconds
    #[opt(long, default_value = "5")]
    time_limit: u64,
    /// number of threads
    #[opt(long, default_value = "1")]
    threads: usize,
    /// problem id
    problem_id: u32,
) -> Result<()> {
    let cl = Client::new();
    let userboard = cl.get_userboard()?;

    let best_score = userboard.problems[(problem_id - 1) as usize].unwrap_or(0.);

    println!("our best score: {}", best_score);

    let problem = Problem::read_from_file(format!("problems/{}.json", problem_id))?;

    let solver = Solver::new(problem_id, problem, time_limit as u64, 42);

    let (score, board) = solver.solve();

    println!("final score: {}", score);

    let solution: Solution = board.try_into().unwrap();

    if score > best_score {
        cl.post_submission(problem_id, solution)?;

        println!("Submitted solution for problem {}!", problem_id);
    }

    Ok(())
}
