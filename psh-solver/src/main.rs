use std::path::PathBuf;

use anyhow::{Result, bail};
use common::{problem::{Problem, Solution, Placement}, evaluate};
use euclid::default::Point2D;
use lyon_geom::LineSegment;
use rand::Rng;

fn generate_random_position(rng: &mut impl Rng, problem: &Problem) -> Vec<Point2D<f64>> {
    let mut pos: Vec<Point2D<f64>> = vec![];
    while pos.len() < problem.musicians.len() {
        let x: f64 = rng.gen_range(problem.stage.min.x + 10f64..problem.stage.max.x - 10f64);
        let y: f64 = rng.gen_range(problem.stage.min.y + 10f64..problem.stage.max.y - 10f64);
        let p = Point2D::new(x, y);
        if pos.iter().any(|&cur| p.distance_to(cur) < 10.) {
            continue;
        }
        pos.push(p);
    }
    pos
}

fn assign_musicians(rng: &mut impl Rng, problem: &Problem, pos: &[Point2D<f64>]) -> Vec<Point2D<f64>> {
    // values[Pos][Inst].
    let mut values: Vec<Vec<f64>> = vec![];
    for i in 0..pos.len() {
        let mut sum = vec![0f64; problem.attendees[0].tastes.len()];
        for a in problem.attendees.iter() {
            let seg = LineSegment {
                from: pos[i],
                to: a.position,
            };
            let mut blocked = false;
            for j in 0..pos.len() {
                if i == j {
                    continue
                }
                if seg.distance_to_point(pos[j]) < 10. {
                    blocked = true;
                    break;
                }
            }
            if !blocked {
                let d = seg.length();
                for (i, taste) in a.tastes.iter().enumerate() {
                    sum[i] += 1_000_000f64 * taste / (d * d);
                }
            }
        }
        values.push(sum);
    }

    let mut idx: Vec<usize> = (0..problem.musicians.len()).into_iter().collect::<Vec<_>>();
    let mut used_list: Vec<bool> = vec![false; problem.musicians.len()];
    let mut placements = vec![Point2D::new(0f64, 0f64); problem.musicians.len()];
    for i in idx {
        let inst = problem.musicians[i];
        let mut max_index = usize::MAX;
        let mut max_score = f64::MIN;
        for (j, u) in used_list.iter().enumerate() {
            if *u {
                continue
            }
            if max_score < values[j][inst] {
                max_index = j;
                max_score = values[j][inst];
            }
        }

        placements[i] = pos[max_index];
        used_list[max_index] = true;
    }
    placements
}

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let args: Vec<String> = std::env::args().collect();
    let problem_id = args[1].parse::<u32>()?;
    let f = PathBuf::from(format!("problems/{}.json", problem_id));
    if !f.is_file() {
        bail!("File not found: {}", f.display());
    }
    let problem = Problem::read_from_file(f)?;
    let pos = generate_random_position(&mut rng, &problem);
    let placements = assign_musicians(&mut rng, &problem, &pos);
    let solution = Solution {
        problem_id,
        placements: placements.iter().map(|p| Placement{position: *p}).collect::<Vec<_>>(),
    };
    let score = evaluate(&problem, &solution);
    println!("{:?}", score);
    Solution::write_to_file(format!("psh-solution/{}-{}.json", problem_id, chrono::Local::now().timestamp()), solution);
    Ok(())
}
