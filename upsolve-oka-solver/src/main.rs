pub mod pretty;
pub mod solver;

use common::{api::Client, evaluate, Solution};
use pretty::pretty;
use solver::Solver;

#[argopt::cmd]
fn main(problem_id: u32, #[opt(long, short, default_value = "500000")] num_iter: usize) {
    let cl = Client::new();

    let problem = cl.get_problem(problem_id).unwrap();

    let mut solver = Solver::new(problem_id, problem.clone(), num_iter);

    let board = solver.solve();

    let solution = board.solution_with_optimized_volume().unwrap();

    let score = evaluate(&problem, &solution);

    eprintln!(
        "Final score: {}, {}",
        pretty(score as i64),
        pretty(board.score() as i64)
    );

    let filename = format!("/tmp/{}_{}.json", problem_id, score);

    eprintln!("Writing to file {filename}");

    Solution::write_to_file(filename, solution).unwrap();
}
