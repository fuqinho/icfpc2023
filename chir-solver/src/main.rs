use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;
use common::{api::Client, board::Board, evaluate, Placement, Problem, RawSolution};
use euclid::default::Point2D;
use indexmap::IndexMap;
use log::{debug, info};
use lyon_geom::point;
use rand::{seq::SliceRandom, Rng};

const SOLVER_NAME: &str = "chir-solver";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    problem_id: u32,
    #[arg(short, long)]
    submit: bool,
    #[arg(long)]
    swap_colors: bool,
    #[arg(long)]
    pick_and_move: bool,
    #[arg(long)]
    initial_solution: Option<PathBuf>,
    #[arg(long, default_value_t = 1.0)]
    step: f64,
    #[arg(long)]
    from_current_best: bool,
}

#[derive(Debug)]
struct Solution {
    pub musicians: IndexMap<(u32, u32), usize>,
}

fn convert_to_real_point(x: u32, y: u32, prob: &Problem) -> Point2D<f64> {
    Point2D::new(
        (x * 10) as f64 + prob.stage.min.x + 10.0,
        (y * 10) as f64 + prob.stage.min.y + 10.0,
    )
}

fn generate_grid_points(prob: &Problem) -> Vec<(u32, u32)> {
    let h = prob.stage.height() as i32;
    let w = prob.stage.width() as i32;
    let sh = (h - 20) / 10 + 1;
    let sw = (w - 20) / 10 + 1;

    let mut grid_points = vec![];
    for x in 0..sw {
        for y in 0..sh {
            grid_points.push((x as u32, y as u32));
        }
    }
    grid_points
}

fn generate_random_solution(prob: &Problem) -> Solution {
    let mut rng = rand::thread_rng();

    let mut possible_points = generate_grid_points(prob);
    possible_points.shuffle(&mut rng);

    let mut sol = IndexMap::new();
    for i in 0..prob.musicians.len() {
        let x = possible_points[i].0;
        let y = possible_points[i].1;
        sol.insert((x, y), prob.musicians[i]);
    }
    Solution { musicians: sol }
}

fn convert_solution(prob: &Problem, sol: &Solution, pid: u32) -> common::Solution {
    let mut musicians_by_inst = HashMap::new();
    for inst in prob.musicians.iter() {
        musicians_by_inst.insert(inst, vec![]);
    }
    for (p, v) in sol.musicians.iter() {
        musicians_by_inst
            .get_mut(v)
            .expect("Should not null")
            .push(convert_to_real_point(p.0, p.1, prob));
    }

    let mut ps = vec![];
    for inst in prob.musicians.iter() {
        ps.push(
            musicians_by_inst
                .get_mut(inst)
                .expect("Should not null")
                .pop()
                .expect("Should not null"),
        );
    }

    common::Solution {
        problem_id: pid,
        solver: SOLVER_NAME.to_owned(),
        placements: ps
            .iter()
            .map(|p| Placement {
                position: p.clone(),
            })
            .collect(),
    }
}

// Hill climb by swap taste

fn hill_climb_swap(
    pid: u32,
    prob: &Problem,
    initial_board: Option<common::Solution>,
) -> Result<common::Solution> {
    let cur = initial_board.unwrap_or(convert_solution(prob, &generate_random_solution(prob), pid));

    let mut board = Board::new(pid, prob.clone(), SOLVER_NAME);
    for (i, placement) in cur.placements.iter().enumerate() {
        board.try_place(i, placement.position)?;
    }

    info!("Initialized");

    loop {
        let mut cur_score = board.score();
        let init_score = board.score();
        let mut updated = false;

        // Trying swap
        for i in 0..board.musicians().len() {
            let (pi, _) = board.musicians()[i].unwrap();
            for j in (i + 1)..board.musicians().len() {
                let (pj, _) = board.musicians()[j].unwrap();
                board.unplace(i);
                board.unplace(j);
                board.try_place(i, pj.to_point())?;
                board.try_place(j, pi.to_point())?;
                if board.score() <= cur_score {
                    board.unplace(i);
                    board.unplace(j);
                    board.try_place(i, pi.to_point())?;
                    board.try_place(j, pj.to_point())?;
                } else {
                    updated = true;
                    cur_score = board.score();
                    break;
                }
            }
        }

        if !updated {
            break;
        }
        info!("Improved {:?} -> {:?}", init_score, board.score());
    }

    board.try_into()
}

fn calc_neighbor(i: usize, p: Point2D<f64>) -> Point2D<f64> {
    const MULT: f64 = 1.0;
    let d: [Point2D<f64>; 8] = [
        point(0., 1.),
        point(0.5, 0.5),
        point(1., 0.),
        point(0.5, -0.5),
        point(0., -1.),
        point(-0.5, -0.5),
        point(-1., 0.),
        point(-0.5, 0.5),
    ];
    point(p.x + MULT * d[i].x, p.y + MULT * d[i].y)
}

fn output_to_results(pid: u32, score: f64, sol: common::Solution) -> Result<()> {
    if !std::path::Path::new("results").is_dir() {
        std::fs::create_dir_all("results")?;
    }
    let output = PathBuf::from(format!("results/{}-{}.json", pid, score as i32));
    common::Solution::write_to_file(output, sol)?;
    Ok(())
}

fn pick_and_move(
    prob: &Problem,
    pid: u32,
    initial_board: Option<common::Solution>,
    step: f64,
) -> Result<common::Solution> {
    let cur = initial_board.unwrap_or(convert_solution(prob, &generate_random_solution(prob), pid));
    let mut board = Board::new(pid, prob.clone(), SOLVER_NAME);
    for i in 0..cur.placements.len() {
        board
            .try_place(i, cur.placements[i].position)
            .expect(&format!(
                "Should be on stage {:?}",
                cur.placements[i].position
            ));
    }
    let mut rng = rand::thread_rng();

    let mut cnt = 0;

    info!("initial = {}", evaluate(prob, &cur));

    let mut max_score = board.score();
    let mut max_board = board.clone();

    const MAX_LOOP: usize = 100;
    loop {
        if max_score < board.score() {
            info!("{}: max updated {}", cnt, board.score());
            max_board = board.clone();
            max_score = board.score();
            let _ = output_to_results(pid, max_score, board.clone().try_into()?);
        }

        // Pick neighbor
        match rng.gen_range(0..=100) {
            0..=10 => {
                let m = rng.gen_range(0..cur.placements.len());
                let p = board.musicians()[m].expect("Should not null");
                let neighbor = rng.gen_range(0..8);
                let cur_s = board.score();
                let np = calc_neighbor(neighbor, p.0.to_point());
                board.unplace(m);
                if board
                    .try_place(rng.gen_range(0..cur.placements.len()), np)
                    .is_ok()
                {
                    if board.score() > cur_s || rng.gen_range(0..=30) == 0 {
                        debug!(
                            "{}: Found good neighbor for {}: {:?} -> {:?}",
                            cnt, m, p, np
                        );
                        continue;
                    }
                    board.unplace(m);
                }
                board.try_place(m, p.0.to_point()).expect(&format!(
                    "Should be on okay {:?}, {:?}",
                    p,
                    p.0.to_point()
                ));
            }
            _ => {
                let m = rng.gen_range(0..cur.placements.len());
                let mut cur_score = board.score();
                let mut next_pos = None;
                let start_x = (prob.stage.min.x + 10.).ceil() as u32;
                let start_y = (prob.stage.min.y + 10.).ceil() as u32;
                let end_x = (prob.stage.max.x - 10.).floor() as u32;
                let end_y = (prob.stage.max.y - 10.).floor() as u32;
                let mut x = start_x as f64;
                while x <= end_x as f64 {
                    let mut y = start_y as f64;
                    while y <= end_y as f64 {
                        let np = point(x as f64, y as f64);
                        let mut new_board = board.clone();
                        new_board.unplace(m);
                        if new_board.can_place(m, np) {
                            new_board.try_place(m, np)?;
                            if new_board.score() > cur_score {
                                cur_score = new_board.score();
                                next_pos = Some(np);
                            }
                        }
                        y += step;
                    }
                    x += step;
                }
                if cur_score > board.score() {
                    info!(
                        "Found new place for {}: {} -> {}",
                        m,
                        board.score(),
                        cur_score
                    );

                    board.unplace(m);
                    board.try_place(m, next_pos.unwrap())?;
                }
            }
        }

        info!("{}/{} done", cnt, MAX_LOOP);
        cnt += 1;
        if cnt >= MAX_LOOP {
            break;
        }
    }

    info!(
        "max = {}, cnt = {}, score = {}",
        max_score,
        cnt,
        max_board.score()
    );
    max_board.try_into()
}

fn get_best_solution(problem_id: u32) -> Result<common::Solution> {
    let url = format!(
        "https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/problems/{problem_id}/best-solution"
    );
    let raw: RawSolution = reqwest::blocking::get(&url)?.json()?;
    Ok(raw.into())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let f = PathBuf::from(format!("../problems/{}.json", args.problem_id));
    if !f.is_file() {
        return Err(anyhow!("File not found: {}", f.display()));
    }
    let problem: Problem = Problem::read_from_file(f)?;

    // Load initial solution if given.
    let initial_solution = if let Some(path) = args.initial_solution {
        let s = std::fs::read_to_string(path)?;
        Some(common::Solution::from(RawSolution::from_json(&s)?))
    } else if args.from_current_best {
        let solution = get_best_solution(args.problem_id).expect("Failed to get best solution");
        Some(solution)
    } else {
        None
    };

    let initial_score = if let Some(ref sol) = initial_solution {
        evaluate(&problem, &sol)
    } else {
        0.0
    };

    let mut sol = convert_solution(
        &problem,
        &generate_random_solution(&problem),
        args.problem_id,
    );
    if args.swap_colors {
        sol = hill_climb_swap(args.problem_id, &problem, initial_solution.clone())?;
    }

    if args.pick_and_move {
        sol = pick_and_move(&problem, args.problem_id, initial_solution, args.step)?;
    }

    let score = evaluate(&problem, &sol);
    info!("score = {:?}", score);

    {
        if !std::path::Path::new("results").is_dir() {
            std::fs::create_dir_all("results")?;
        }
        let output = PathBuf::from(format!("results/{}-{}.json", args.problem_id, score));
        common::Solution::write_to_file(output, sol.clone())?;
    }

    if args.submit && score > initial_score {
        let c = Client::new();
        c.post_submission(args.problem_id, sol)?;
    } else {
        info!("Skip submitting because of no improvement");
    }

    Ok(())
}
