#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    clippy::needless_range_loop
)]

mod solver;

use std::{fs::File, path::Path};

use anyhow::Result;
use common::{api::Client, evaluate, Solution};

use crate::solver::Solver;

// use crate::solver::Solver;

#[argopt::cmd]
fn main(
    problem_id: u32,
    #[opt(short, long, default_value = "")] out: String,
    #[opt(short, long, default_value = "")] profile: String,
) -> Result<()> {
    let gurad = if !profile.is_empty() {
        Some(
            pprof::ProfilerGuardBuilder::default()
                .frequency(1000)
                .blocklist(&["libc", "libgcc", "pthread", "vdso"])
                .build()
                .unwrap(),
        )
    } else {
        None
    };

    let cl = Client::new();
    let userboard = cl.get_userboard()?;

    let best_score = userboard.problems[(problem_id - 1) as usize].unwrap_or(0.);

    eprintln!("our best score: {}", best_score);

    let problem = cl.get_problem(problem_id)?;

    let mut solver = Solver::new(problem_id, problem.clone());

    let (_score, board) = solver.solve();

    let solution: Solution = board.solution_with_optimized_volume().unwrap();

    let eval_score = evaluate(&problem, &solution);

    eprintln!("final score: {}", eval_score);

    if eval_score > best_score {
        cl.post_submission(problem_id, solution.clone())?;

        eprintln!("Submitted solution for problem {}!", problem_id);
    }

    if out != "" {
        eprintln!("Writing solution to {}", out);
        Solution::write_to_file(out, solution)?;
    }

    if let Some(guard) = gurad {
        if let Ok(report) = guard.report().build() {
            eprintln!("Writing profile to {}", profile);

            let file = File::create(profile).unwrap();
            report.flamegraph(file).unwrap();
        };
    }

    Ok(())
}
