use std::path::PathBuf;

use anyhow::{bail, Result};
use common::evaluate;
use common::{Placement, Problem, Solution};
use euclid::default::Point2D;
use lyon_geom::LineSegment;

fn solver_10(problem: &Problem) -> Solution {
    let eps = 1e-9;
    let height_unit = 3f64.sqrt() * 5f64 + eps;
    let width_unit = 1f64;

    let offset = Point2D::new(
        problem.stage.min.x + 10.,
        problem.stage.min.y + height_unit * 10. + 10.,
    );
    let mut current_col = 0;
    let mut current_row = 0;
    let mut count_3 = 0;
    let mut placements = vec![];
    for p in problem.musicians.iter() {
        if *p == 3 {
            let row = count_3 / 2;
            if row <= 9 {
                placements.push(Point2D::new(
                    offset.x
                        + ((if count_3 % 2 == 0 { 0 } else { (13 - row) * 10 } + row * 5) as f64)
                            * width_unit,
                    offset.y - row as f64 * height_unit,
                ));
            } else {
                let row = 9;
                placements.push(Point2D::new(
                    offset.x + ((5 * row) as f64 + ((count_3 - 19) * 10) as f64) * width_unit,
                    offset.y - row as f64 * height_unit,
                ));
            }
            count_3 += 1;
            continue;
        }

        placements.push(Point2D::new(
            offset.x + (((current_col + 1) * 10 + current_row * 5) as f64) * width_unit,
            offset.y - current_row as f64 * height_unit,
        ));
        current_col += 1;
        if current_col >= 12 - current_row {
            current_col = 0;
            current_row += 1;
        }
    }

    Solution {
        problem_id: 10,
        placements: placements
            .into_iter()
            .map(|p| Placement { position: p })
            .collect::<Vec<_>>(),
    }
}

fn validate(solution: &Solution) {
    for (i1, p1) in solution.placements.iter().enumerate() {
        for (i2, p2) in solution.placements.iter().enumerate() {
            if i1 == i2 {
                continue;
            }
            let seg = LineSegment {
                from: p1.position,
                to: p2.position,
            };
            if seg.square_length() < 100.0 {
                eprintln!("Too close: p[{:}] = {:?}, p[{:}] = {:?}", i1, p1, i2, p2);
            }
        }
    }
}

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let args: Vec<String> = std::env::args().collect();
    let problem_id = args[1].parse::<u32>()?;
    let is_estimate = if args.len() >= 3 && args[2] == "estimate" {
        true
    } else {
        false
    };
    let f = PathBuf::from(format!("problems/{}.json", problem_id));
    if !f.is_file() {
        bail!("File not found: {}", f.display());
    }
    let problem = Problem::read_from_file(f)?;

    let solution = match problem_id {
        _ if is_estimate => common::estimate(problem_id, &problem).1,
        10 => solver_10(&problem),
        _ => bail!("no solution for {:}", problem_id),
    };
    validate(&solution);
    println!("score: {:?}", evaluate(&problem, &solution));
    Solution::write_to_file(
        format!(
            "psh-solution/{}-{}-manual.json",
            problem_id,
            chrono::Local::now().timestamp()
        ),
        solution,
    )?;
    Ok(())
}
