use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;
use common::{similarity, Problem, Solution};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    problem_id: u32,
    #[arg(long)]
    solution1: PathBuf,
    #[arg(long)]
    solution2: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut problem_file = PathBuf::from(format!("../problems/{}.json", args.problem_id));
    if !problem_file.is_file() {
        problem_file = PathBuf::from(format!("./problems/{}.json", args.problem_id));
        if !problem_file.is_file() {
            bail!("File not found: {}", problem_file.display());
        }
    }
    let problem = Problem::read_from_file(problem_file)?;
    if !args.solution1.is_file() {
        bail!("File not found: {}", args.solution1.display());
    }
    let solution1 = Solution::read_from_file(args.solution1)?;
    if !args.solution2.is_file() {
        bail!("File not found: {}", args.solution2.display());
    }
    let solution2 = Solution::read_from_file(args.solution2)?;
    println!("similarity = {}", similarity(problem, solution1, solution2));
    Ok(())
}
