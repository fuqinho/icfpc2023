use std::{fs::read_to_string, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;
use common::{api::Client, evaluate, fixup_volumes, Problem, RawProblem, RawSolution};
use fuqinho_solver::sa::{SAConfig, solve_sa};
use fuqinho_solver::solve;
use thousands::Separable;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    problem_id: usize,
    #[arg(long)]
    force_submit: bool,
    #[arg(long)]
    sa: bool,
    #[arg(long)]
    iterations: Option<usize>,
    #[arg(long)]
    initial_temp: Option<f64>,
    #[arg(long)]
    solutions_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let problem_id = args.problem_id;

    // Read the problem json.
    let f = PathBuf::from(format!("problems/{}.json", problem_id));
    if !f.is_file() {
        return Err(anyhow!("File not found: {}", f.display()));
    }
    let json = read_to_string(f)?;
    let raw_problem: RawProblem = serde_json::from_str(&json)?;
    let problem = Problem::from(raw_problem);

    // Solve the problem.
    let mut solution = if args.sa {
        let config = SAConfig {
            num_iterations: args.iterations,
            initial_temperature: args.initial_temp,
            solutions_dir: args.solutions_dir,
        };
        solve_sa(&problem, problem_id as u32, &config)
    } else {
        solve(&problem, problem_id)
    };
    solution = fixup_volumes(&problem, &solution);
    let score = evaluate(&problem, &solution);
    eprintln!("best score: {}", score.separate_with_commas());

    // Write the solution to file.
    let solution_to_write = solution.clone();
    let solution_json = serde_json::to_string(&RawSolution::from(solution_to_write))?;
    let output = PathBuf::from(format!("{}-{}.json", args.problem_id, score));
    std::fs::write(output, solution_json)?;

    // Submit the solution if it is our best.
    let api_client = Client::new();
    let user_board = api_client.get_userboard()?;
    let our_best_score = user_board.problems[problem_id - 1].unwrap_or(0.0);
    if score > our_best_score || args.force_submit {
        api_client.post_submission(problem_id as u32, solution)?;
        eprintln!(
            "Score submitted. {}: {} -> {}",
            problem_id, our_best_score, score
        );
    }

    Ok(())
}
