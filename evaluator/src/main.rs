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
    let problem_file = PathBuf::from(format!("../problems/{}.json", args.problem_id));
    if !problem_file.is_file() {
        bail!("File not found: {}", problem_file.display());
    }
    let problem = Problem::read_from_file(problem_file)?;
    if !args.solution.is_file() {
        bail!("File not found:{}", args.solution.display());
    }
    let solution_str = std::fs::read_to_string(args.solution)?;
    let solution_raw = RawSolution::from_json(&solution_str)?;
    let solution = Solution::from(solution_raw);

    println!("score = {}", evaluate(&problem, &solution));
    // Evaluate by board
    let mut board = Board::new(args.problem_id, problem, "N/A");
    for (i, placement) in solution.placements.iter().enumerate() {
        board.try_place(i, placement.position)?;
    }
    println!("score by board = {}", board.score());

    Ok(())
}
