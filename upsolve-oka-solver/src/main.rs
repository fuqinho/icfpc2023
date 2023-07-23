pub mod output;
pub mod params;
pub mod pretty;
pub mod solver;
pub mod solver2;
pub mod solver3;

use std::{
    fs::{read_to_string, File},
    io::Write,
};

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;

use common::{evaluate, Problem, Solution};
use pprof::protos::Message;
use pretty::pretty;
use solver::Solver;

use crate::{params::Params, solver2::Solver2};

const PARAMS: &str = include_str!("../params.json");

#[argopt::cmd]
fn main(
    problem_id: u32,
    #[opt(long, short, default_value = "5000000")] num_iter: usize,
    #[opt(long, short)] profile: bool,
    #[opt(long, default_value = "")] params: String,
    #[opt(long, default_value = "")] output: String,
    #[opt(long)] quiet: bool,
    #[opt(long, default_value = "")] initial_solution: String,
    #[opt(long, short, default_value = "1")] version: usize, // solver version
) {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(
            None,
            if quiet {
                LevelFilter::Warn
            } else {
                LevelFilter::Info
            },
        )
        .init();

    let problem = Problem::read_from_file(&format!("problems/{}.json", problem_id)).unwrap();

    let params_str = if params.is_empty() {
        PARAMS.to_string()
    } else {
        read_to_string(params).unwrap()
    };

    let params: Params = serde_json::from_str(&params_str).unwrap();

    let initial_solution = if !initial_solution.is_empty() {
        Some(Solution::read_from_file(initial_solution).unwrap())
    } else {
        None
    };

    let guard = if profile {
        Some(pprof::ProfilerGuardBuilder::default().build().unwrap())
    } else {
        None
    };

    let board = match version {
        1 => Solver::new(
            problem_id,
            problem.clone(),
            num_iter,
            params,
            initial_solution,
        )
        .solve(),
        2 => Solver2::new(
            problem_id,
            problem.clone(),
            num_iter,
            params,
            initial_solution,
        )
        .solve(),
        3 => solver3::solve(
            problem_id,
            problem.clone(),
            num_iter,
            params,
            initial_solution,
        ),
        _ => panic!("Unknown solver version: {}", version),
    };

    if let Some(guard) = guard {
        if let Ok(report) = guard.report().build() {
            println!("Writing /tmp/profile.pb");
            let mut file = File::create("/tmp/profile.pb").unwrap();
            let profile = report.pprof().unwrap();

            let mut content = Vec::new();
            profile.write_to_vec(&mut content).unwrap();
            file.write_all(&content).unwrap();

            println!("Writing /tmp/profile.svg");
            let mut file = File::create("/tmp/profile.svg").unwrap();
            report.flamegraph(&mut file).unwrap();
        };
    }

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

    if !output.is_empty() {
        serde_json::to_writer(File::create(output).unwrap(), &output::Output { score }).unwrap();
    }
}
