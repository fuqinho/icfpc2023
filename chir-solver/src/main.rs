use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;
use common::{api::Client, board::Board, evaluate, Placement, Problem, RawSolution};
use euclid::default::Point2D;
use indexmap::IndexMap;
use log::{debug, info};
use lyon_geom::{point, LineSegment};
use rand::{seq::SliceRandom, Rng};

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

fn validate_solution(prob: &Problem, sol: &Solution) -> bool {
    for s in sol.musicians.keys() {
        let p = convert_to_real_point(s.0, s.1, prob);
        if !prob.stage.contains(p) {
            return false;
        }
    }
    true
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
        placements: ps
            .iter()
            .map(|p| Placement {
                position: p.clone(),
            })
            .collect(),
    }
}

// Hill climb by swap taste

fn swap_taste(sol: &mut Solution, i: usize, j: usize) {
    let w = sol
        .musicians
        .get_index(i)
        .expect("Should not null")
        .1
        .clone();
    let v = sol
        .musicians
        .get_index(j)
        .expect("Should not null")
        .1
        .clone();
    {
        *sol.musicians.get_index_mut(i).expect("Should not null").1 = v;
    }
    {
        *sol.musicians.get_index_mut(j).expect("Should not null").1 = w;
    }
}

fn score_by_musician(i: usize, cur: &Solution, blocked: &Vec<Vec<bool>>, prob: &Problem) -> f64 {
    let mut s: f64 = 0.;
    let v = cur.musicians.get_index(i).expect("Should exist");
    let taste = *v.1;
    let p = convert_to_real_point(v.0 .0, v.0 .1, prob);
    for j in 0..prob.attendees.len() {
        if blocked[i][j] {
            continue;
        }
        let pa = prob.attendees[j].position;
        let d2 = (pa.x - p.x) * (pa.x - p.x) + (pa.y - p.y) * (pa.y - p.y);
        s += (1000000f64 * prob.attendees[j].tastes[taste] / d2).ceil();
    }
    s
}

fn hill_climb_swap(prob: &Problem) -> Solution {
    let mut cur = generate_random_solution(prob);

    assert!(validate_solution(prob, &cur));

    let mut blocked = vec![vec![false; prob.attendees.len()]; cur.musicians.len()];
    for i in 0..cur.musicians.len() {
        for j in 0..prob.attendees.len() {
            let pt = cur.musicians.get_index(i).expect("Should not null").0;
            let seg = LineSegment {
                from: prob.attendees[j].position,
                to: convert_to_real_point(pt.0, pt.1, prob),
            };
            for k in 0..cur.musicians.len() {
                if i == k {
                    continue;
                }
                let kt = cur.musicians.get_index(k).expect("Should not null").0;
                if seg.square_distance_to_point(convert_to_real_point(kt.0, kt.1, prob)) <= 25. {
                    blocked[i][j] = true;
                }
            }
        }
    }
    info!("block map is calculated");

    let mut score_by_m = vec![0.; cur.musicians.len()];
    for i in 0..cur.musicians.len() {
        score_by_m[i] = score_by_musician(i, &cur, &blocked, prob);
    }
    info!("score by musician is calculated");

    loop {
        let cur_score: f64 = score_by_m.iter().sum();
        let mut s = cur_score;
        // Swap taste
        for i in 0..cur.musicians.len() {
            for j in (i + 1)..cur.musicians.len() {
                swap_taste(&mut cur, i, j);
                score_by_m[i] = score_by_musician(i, &cur, &blocked, prob);
                score_by_m[j] = score_by_musician(j, &cur, &blocked, prob);

                let new_score: f64 = score_by_m.iter().sum();
                if new_score <= s {
                    swap_taste(&mut cur, i, j);
                    score_by_m[i] = score_by_musician(i, &cur, &blocked, prob);
                    score_by_m[j] = score_by_musician(j, &cur, &blocked, prob);
                } else {
                    debug!("Updated {} -> {}", s, new_score);
                    s = new_score;
                }
            }
        }

        if cur_score == s {
            break;
        }
        info!("Improved {:?} -> {:?}", cur_score, s);
    }

    cur
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
    let mut board = Board::new(pid, prob.clone());
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
                let np = calc_neighbor(neighbor, p.to_point());
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
                board.try_place(m, p.to_point()).expect(&format!(
                    "Should be on okay {:?}, {:?}",
                    p,
                    p.to_point()
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

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let f = PathBuf::from(format!("../problems/{}.json", args.problem_id));
    if !f.is_file() {
        return Err(anyhow!("File not found: {}", f.display()));
    }
    let problem: Problem = Problem::read_from_file(f)?;

    // Load initial solution if given.
    let initial_solution: Option<common::Solution> = args.initial_solution.map(|path| {
        let content = std::fs::read_to_string(path).expect("File not found");
        let raw_sol = RawSolution::from_json(&content).expect("Failed to parse");
        common::Solution::from(raw_sol)
    });

    let mut sol = generate_random_solution(&problem);
    if args.swap_colors {
        sol = hill_climb_swap(&problem);
    }

    let mut raw_sol = convert_solution(&problem, &sol, args.problem_id);
    if args.pick_and_move {
        raw_sol = pick_and_move(&problem, args.problem_id, initial_solution, args.step)?;
    }

    let score = evaluate(&problem, &raw_sol);
    info!("score = {:?}", score);

    {
        if !std::path::Path::new("results").is_dir() {
            std::fs::create_dir_all("results")?;
        }
        let output = PathBuf::from(format!("results/{}-{}.json", args.problem_id, score));
        common::Solution::write_to_file(output, raw_sol)?;
    }

    if args.submit {
        let c = Client::new();
        c.post_submission(
            args.problem_id,
            convert_solution(&problem, &sol, args.problem_id),
        )?;
    }

    Ok(())
}
