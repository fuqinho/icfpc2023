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

fn get_recent_solution() -> Result<Solution> {
    let s = std::fs::read_to_string("results/recent.json")?;
    let raw: RawSolution = serde_json::from_str(&s)?;
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
    #[opt(long, default_value = "1.0")]
    limit_temp: f64,
    /// prune far atendees
    #[opt(long)]
    prune_far: Option<f64>,
    /// start from current best solution
    #[opt(
        long,
        conflicts_with = "better-initial",
        conflicts_with = "from-recent",
        conflicts_with = "initial-solution"
    )]
    from_current_best: bool,
    /// start from recent solution
    #[opt(
        long,
        conflicts_with = "from-current-best",
        conflicts_with = "better-initial",
        conflicts_with = "initial-solution"
    )]
    from_recent: bool,
    /// from better initial solution
    #[opt(
        long,
        conflicts_with = "from-current-best",
        conflicts_with = "from-recent",
        conflicts_with = "initial-solution"
    )]
    better_initial: bool,
    /// specify initial solution
    #[opt(
        long,
        conflicts_with = "from-current-best",
        conflicts_with = "better-initial",
        conflicts_with = "from-recent"
    )]
    initial_solution: Option<PathBuf>,
    /// annealing specify taste
    #[opt(long)]
    taste: Option<usize>,
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

    let param = format!(
        "{}sec,{}ths{}",
        time_limit,
        threads,
        if initial_solution.is_some() {
            ",w/init"
        } else if from_current_best || from_recent {
            ",cont"
        } else {
            ",scratch"
        }
    );

    let initial_solution: Option<Solution> = if let Some(path) = initial_solution {
        let s = std::fs::read_to_string(path)?;
        let raw_solution: RawSolution = serde_json::from_str(&s)?;
        Some(raw_solution.into())
    } else if from_current_best {
        let solution = get_best_solution(problem_id)?;
        Some(solution)
    } else if from_recent {
        let solution = get_recent_solution()?;
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
        taste,
        param,
    };

    let solution = saru::annealing(
        &solver,
        &saru::AnnealingOptions {
            time_limit,
            limit_temp,
            restart: 0,
            silent: false,
            header: format!("{problem_id}: "),
        },
        rand::thread_rng().gen(),
        threads,
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
    if let Some(initial_solution) = &initial_solution {
        let initial_score = common::evaluate(&orig_problem, initial_solution);
        eprintln!(
            "Score improvement: {} ({:.3}%)",
            acc_score - initial_score,
            (acc_score - initial_score) / initial_score * 100.0
        );
    }
    eprintln!("Musicians:        {}", solver.problem.musicians.len());
    eprintln!("Atendees:         {}", solver.problem.attendees.len());
    eprintln!("Stage area:       {}", solver.problem.stage.area());

    let raw_solution = RawSolution::from(solution.clone());

    {
        if !std::path::Path::new("results").is_dir() {
            std::fs::create_dir_all("results")?;
        }
        let file_name = format!("results/sol-{problem_id:03}-{}.json", acc_score);
        let s = format!("{}", serde_json::json!(raw_solution));
        std::fs::write(file_name, &s)?;
        std::fs::write("results/recent.json", s)?;
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
