use std::{fs::read_to_string, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;
use common::{Problem, RawProblem, RawSolution};

use fuqinho_solver::solve;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    problem_id: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let f = PathBuf::from(format!("../problems/{}.json", args.problem_id));
    if !f.is_file() {
        return Err(anyhow!("File not found: {}", f.display()));
    }
    let json = read_to_string(f)?;
    let raw_problem: RawProblem = serde_json::from_str(&json)?;
    let problem = Problem::from(raw_problem);

    let mut solution = solve(&problem);

    solution.problem_id = args.problem_id;
    let solution_json = serde_json::to_string(&RawSolution::from(solution))?;
    let output = PathBuf::from(format!("{}-out.json", args.problem_id));
    std::fs::write(output, solution_json)?;

    Ok(())
}

