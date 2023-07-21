use common::Problem;
use lyon_geom::{Box2D, LineSegment};

use super::types::P;

#[derive(Clone, Debug)]
pub struct ExtProblem {
    pub problem: Problem,
    pub walls: Vec<LineSegment<f64>>,
}

pub fn split_problem(problem: Problem, side: f64) -> Vec<ExtProblem> {
    if problem.stage.min.x > 0. || problem.stage.min.y > 0. {
        panic!("Unsupported problem");
    }
    let margin = P::new(5., 5.);
    let inner_box = Box2D::new(problem.stage.min + margin, problem.stage.max - margin);

    let nw = (inner_box.size().width / side).ceil() as usize;
    let nh = (inner_box.size().height / side).ceil() as usize;

    let w = inner_box.size().width / nw as f64 - 1e-9;
    let h = inner_box.size().height / nh as f64 - 1e-9;

    let dw = P::new(w, 0.);
    let dh = P::new(0., h);

    let mut mini_problems = vec![];

    for i in 0..nw {
        for j in 0..nh {
            if i < nw - 1 && j < nh - 1 {
                continue;
            }

            let ll = inner_box.min + dw * i as f64 + dh * j as f64;
            let ur = ll + dw + dh;

            let mini_stage = Box2D::new(ll - margin, ur + margin);

            let ds = [(0, 0), (0, 1), (1, 1), (1, 0)];

            let mut walls = vec![];
            for k in 0..4 {
                let (di1, dj1) = ds[k];
                let (di2, dj2) = ds[(k + 1) % 4];

                let (i1, j1) = (i + di1, j + dj1);
                let (i2, j2) = (i + di2, j + dj2);

                if i1 == nw && i2 == nw || j1 == nh && j2 == nh {
                    continue;
                }

                let p1 = inner_box.min + dw * i1 as f64 + dh * j1 as f64;
                let p2 = inner_box.min + dw * i2 as f64 + dh * j2 as f64;

                let wall = LineSegment { from: p1, to: p2 };

                walls.push(wall);
            }

            mini_problems.push(ExtProblem {
                problem: Problem {
                    stage: mini_stage,
                    ..problem.clone()
                },
                walls,
            });
        }
    }

    mini_problems
}
