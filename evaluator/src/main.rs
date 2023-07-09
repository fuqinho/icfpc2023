use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;
use common::{board::Board, evaluate, Problem, RawSolution, Solution};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    problem_id: u32,
    #[arg(short, long)]
    solution: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut problem_file = PathBuf::from(format!("../problems/{}.json", args.problem_id));
    if !problem_file.is_file() {
        // Look at the current dir, too, to support "cargo run --bin evaluator" at top dir.
        problem_file = PathBuf::from(format!("./problems/{}.json", args.problem_id));
        if !problem_file.is_file() {
            bail!("File not found: {}", problem_file.display());
        }
    }
    let problem = Problem::read_from_file(problem_file)?;
    if !args.solution.is_file() {
        bail!("File not found:{}", args.solution.display());
    }
    let solution = Solution::read_from_file(args.solution)?;

    println!("score = {}", evaluate(&problem, &solution));
    // Evaluate by board
    let mut board = Board::new(args.problem_id, problem, "N/A");
    for (i, placement) in solution.placements.iter().enumerate() {
        board.try_place(i, placement.position)?;
    }
    for (i, volume) in solution.volumes.iter().enumerate() {
        board.set_volume(i, *volume);
    }
    println!("score by board = {}", board.score());

    Ok(())
}
