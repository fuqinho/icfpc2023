use anyhow::Result;
use common::{api::Client, RawSolution, Solution};
use rand::Rng;
use std::path::PathBuf;

use tanakh_solver::solver::{post_process, pre_process, Solver2};

fn get_best_solution(problem_id: u32) -> Result<Solution> {
    let url = format!(
        "https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/problems/{problem_id}/best-solution"
    );
    let raw: RawSolution = reqwest::blocking::get(&url)?.json()?;
    Ok(raw.into())
}

#[argopt::cmd]
fn main(
    /// time limit in seconds
    #[opt(long, default_value = "10.0")]
    time_limit: f64,
    /// number of threads
    #[opt(long, default_value = "1")]
    threads: usize,
    /// specify start temperature
    #[opt(long)]
    start_temp: Option<f64>,
    /// specify limit temerature
    #[opt(long, default_value = "0.1")]
    limit_temp: f64,
    /// prune far atendees
    #[opt(long)]
    prune_far: Option<f64>,
    /// start from current best solution
    #[opt(
        long,
        conflicts_with = "better-initial",
        conflicts_with = "initial-solution"
    )]
    from_current_best: bool,
    /// from better initial solution
    #[opt(
        long,
        conflicts_with = "from-current-best",
        conflicts_with = "initial-solution"
    )]
    better_initial: bool,
    /// specify initial solution
    #[opt(
        long,
        conflicts_with = "from-current-best",
        conflicts_with = "better-initial"
    )]
    initial_solution: Option<PathBuf>,
    /// do not submit
    #[opt(long)]
    no_submit: bool,
    /// problem id
    problem_id: u32,
) -> Result<()> {
    let client = Client::new();

    // let problem = get_problem_from_file(problem_id)?;
    let mut problem = client.get_problem(problem_id)?;

    eprintln!("Musicians: {}", problem.musicians.len());
    eprintln!("Atendees:  {}", problem.attendees.len());

    let orig_problem = problem.clone();

    pre_process(&mut problem, prune_far);

    let initial_solution: Option<Solution> = if let Some(path) = initial_solution {
        let s = std::fs::read_to_string(path)?;
        let raw_solution: RawSolution = serde_json::from_str(&s)?;
        Some(raw_solution.into())
    } else if from_current_best {
        let solution = get_best_solution(problem_id)?;
        Some(solution)
    } else {
        None
    };

    eprintln!("Solving...");

    let solver = Solver2 {
        problem_id,
        problem: problem.clone(),
        start_temp,
        better_initial,
        initial_solution,
    };

    let solution = saru::annealing(
        &solver,
        &saru::AnnealingOptions {
            time_limit,
            limit_temp,
            restart: 0,
            threads,
            silent: false,
            header: format!("{problem_id}: "),
        },
        rand::thread_rng().gen(),
    );

    let Some(mut solution) = solution.solution else {
        anyhow::bail!("Valid solution not found")
    };

    post_process(problem_id, &problem, &mut solution);

    solution.problem_id = problem_id;

    let acc_score = common::evaluate(&orig_problem, &solution);

    eprintln!("Statistics:");
    eprintln!("Problem ID:       {}", problem_id);
    eprintln!("Score:            {}", acc_score);
    eprintln!("Musicians:        {}", solver.problem.musicians.len());
    eprintln!("Atendees:         {}", solver.problem.attendees.len());
    eprintln!("Stage area:       {}", solver.problem.stage.area());

    let raw_solution = RawSolution::from(solution.clone());

    {
        if !std::path::Path::new("results").is_dir() {
            std::fs::create_dir_all("results")?;
        }
        let file_name = format!("results/sol-{problem_id:03}-{}.json", acc_score);
        std::fs::write(file_name, format!("{}", serde_json::json!(raw_solution)))?;
    }

    if acc_score <= 0.0 {
        anyhow::bail!("Positive score not found");
    }

    if !no_submit {
        let resp = client.post_submission(problem_id, solution)?;
        eprintln!("Submitted: {}", resp.0);
    }

    Ok(())
}
