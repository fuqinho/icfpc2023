pub mod pretty;
pub mod solver;

use std::{fs::File, io::Write};

use common::{api::Client, evaluate, Solution};
use pprof::protos::Message;
use pretty::pretty;
use solver::Solver;

#[argopt::cmd]
fn main(
    problem_id: u32,
    #[opt(long, short, default_value = "500000")] num_iter: usize,
    #[opt(long, short)] profile: bool,
) {
    let cl = Client::new();

    let problem = cl.get_problem(problem_id).unwrap();

    let mut solver = Solver::new(problem_id, problem.clone(), num_iter);

    let guard = if profile {
        Some(
            pprof::ProfilerGuardBuilder::default()
                .frequency(100)
                .blocklist(&["libc", "libgcc", "pthread", "vdso"])
                .build()
                .unwrap(),
        )
    } else {
        None
    };

    let board = solver.solve();

    if let Some(guard) = guard {
        if let Ok(report) = guard.report().build() {
            println!("Writing /tmp/profile.pb");
            let mut file = File::create("/tmp/profile.pb").unwrap();
            let profile = report.pprof().unwrap();

            let mut content = Vec::new();
            profile.write_to_vec(&mut content).unwrap();
            file.write_all(&content).unwrap();
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
}
