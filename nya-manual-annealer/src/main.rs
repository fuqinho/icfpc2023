use std::sync::mpsc::Receiver;

use anyhow::Result;
use common::{api::Client, RawSolution, Solution};

use tanakh_solver::solver::{Solver2, State2};
use thousands::Separable;

fn get_best_solution(problem_id: u32) -> Result<Solution> {
    let url = format!(
        "https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/problems/{problem_id}/best-solution"
    );
    let raw: RawSolution = reqwest::blocking::get(&url)?.json()?;
    Ok(raw.into())
}

struct NonBlockingStdinReader {
    rx: Receiver<u8>,
}

impl NonBlockingStdinReader {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let getch = getch::Getch::new();
            while let Ok(ch) = getch.getch() {
                if tx.send(ch).is_err() {
                    break;
                }
            }
        });
        Self { rx }
    }

    pub fn read(&self) -> Option<u8> {
        self.rx.try_recv().ok()
    }
}

#[argopt::cmd]
fn main(
    /// problem id
    problem_id: u32,
) -> Result<()> {
    let client = Client::new();

    let stdin = NonBlockingStdinReader::new();

    let problem = client.get_problem(problem_id)?;
    let initial_solution = get_best_solution(problem_id)?;
    let solver_name = if initial_solution.solver.ends_with("+nya") {
        initial_solution.solver.clone()
    } else {
        format!("{}+nya", &initial_solution.solver)
    };
    let mut state = State2::new(&initial_solution, &problem, &solver_name, false);

    let mut current_temp = 0.0001;

    let solver = Solver2 {
        problem_id,
        problem: problem.clone(),
        start_temp: Some(current_temp),
        better_initial: false,
        initial_solution: None,
        taste: None,
        param: String::new(),
        use_visibility: false,
    };

    let mut best_solution = initial_solution;
    let mut best_score = common::evaluate(&problem, &best_solution);
    let mut submitted_score = best_score;
    let mut best_updated = false;

    eprint!(
        r"
===========================================
職人の手のぬくもりが伝わる焼きなまし器
操作方法:
  w 温度を *= 10
  s 温度を /= 10
  r 最適解を思い出す
  x 最適解をsubmit
===========================================
"
    );

    loop {
        let options = saru::AnnealingOptions {
            time_limit: 1.0,
            limit_temp: current_temp,
            restart: 0,
            silent: true,
            header: String::new(),
        };
        let result = saru::annealing_single_thread(None, &solver, &options, 283, state);
        let mut estimated_score = -result.score;
        if let Some(solution) = result.solution {
            if estimated_score > best_score {
                let accurate_score = common::evaluate(&problem, &solution);
                if accurate_score > best_score {
                    best_score = accurate_score;
                    best_solution = solution;
                    best_updated = true;
                }
            }
        }
        state = result.state;
        while let Some(c) = stdin.read() {
            match c {
                b'w' => {
                    current_temp *= 10.0;
                }
                b's' => {
                    current_temp /= 10.0;
                }
                b'r' => {
                    state = State2::new(&best_solution, &problem, &solver_name, false);
                    estimated_score = best_score;
                }
                b'x' => {
                    if !best_updated {
                        break;
                    }
                    let raw_solution = RawSolution::from(best_solution.clone());
                    {
                        if !std::path::Path::new("results").is_dir() {
                            std::fs::create_dir_all("results")?;
                        }
                        let file_name =
                            format!("results/sol-{problem_id:03}-{}.json", estimated_score);
                        let s = format!("{}", serde_json::json!(raw_solution));
                        std::fs::write(file_name, &s)?;
                        std::fs::write("results/recent.json", s)?;
                    }
                    client
                        .post_submission(problem_id, best_solution.clone())
                        .expect("Submit failed");
                    submitted_score = best_score;
                    best_updated = false;
                }
                _ => {}
            }
        }
        let best_updated_marker = if best_updated {
            format!(
                " *** new best: +{}",
                (best_score - submitted_score).separate_with_commas()
            )
        } else {
            String::new()
        };
        eprintln!(
            "[{}] score = {} / best = {} / temp = {}{}",
            problem_id,
            estimated_score.separate_with_commas(),
            submitted_score.separate_with_commas(),
            current_temp,
            best_updated_marker
        );
    }
}
