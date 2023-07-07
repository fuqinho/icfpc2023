use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;
use common::{api::Client, evaluate, Placement, Problem, RawProblem, RawSolution};
use euclid::default::Point2D;
use indexmap::IndexMap;
use log::{debug, info};
use lyon_geom::LineSegment;
use rand::seq::SliceRandom;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    problem_id: u32,
    #[arg(short, long)]
    submit: bool,
    #[arg(short, long)]
    swap_colors: bool,
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

fn generate_random_solution(prob: &Problem) -> Solution {
    let mut rng = rand::thread_rng();

    let h = prob.stage.height() as i32;
    let w = prob.stage.width() as i32;
    let sh = (h - 20) / 10 + 1;
    let sw = (w - 20) / 10 + 1;

    let mut possible_points = vec![];
    for x in 0..sw {
        for y in 0..sh {
            possible_points.push((x as u32, y as u32));
        }
    }
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

fn score(prob: &Problem, sol: &Solution, pid: u32) -> f64 {
    let raw_sol = convert_solution(prob, sol, pid);
    evaluate(prob, &raw_sol)
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

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let f = PathBuf::from(format!("../problems/{}.json", args.problem_id));
    if !f.is_file() {
        return Err(anyhow!("File not found: {}", f.display()));
    }
    let json = read_to_string(f)?;
    let raw_problem: RawProblem = serde_json::from_str(&json)?;
    let problem: Problem = Problem::from(raw_problem);

    let mut sol = generate_random_solution(&problem);
    if !args.swap_colors {
        sol = hill_climb_swap(&problem);
    }
    info!("score = {:?}", score(&problem, &sol, args.problem_id));

    let raw_sol = convert_solution(&problem, &sol, args.problem_id);
    let raw_sol = serde_json::to_string(&RawSolution::from(raw_sol))?;
    let output = PathBuf::from(format!("{}-out.json", args.problem_id));
    std::fs::write(output, raw_sol)?;

    if args.submit {
        let c = Client::new();
        c.post_submission(
            args.problem_id,
            convert_solution(&problem, &sol, args.problem_id),
        )?;
    }

    Ok(())
}
