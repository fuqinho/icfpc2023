use common::{geom::rotate90, Pillar, Problem};
use lyon_geom::{Box2D, LineSegment, Vector};

use super::types::P;

#[derive(Clone, Debug)]
pub struct ExtProblem {
    pub problem: Problem,
    pub walls: Vec<LineSegment<f64>>,
    pub extra_pillars: Vec<Pillar>,

    pub is_top: bool,
    pub is_right: bool,
}

impl ExtProblem {
    pub fn contains(&self, p: P) -> bool {
        let mut cell = self.problem.stage;
        cell.min += Vector::new(10., 10.);
        cell.max -= Vector::new(10., 10.);
        cell.max += Vector::new(1e-12, 1e-12);

        cell.contains(p.to_point())
    }
}

pub fn split_problem(problem: Problem, side: f64) -> Vec<ExtProblem> {
    let margin = P::new(5., 5.);
    let inner_box = Box2D::new(problem.stage.min + margin, problem.stage.max - margin);

    let nw = (inner_box.size().width / side).ceil() as usize;
    let nh = (inner_box.size().height / side).ceil() as usize;

    let w = inner_box.size().width / nw as f64 - 1e-9;
    let h = inner_box.size().height / nh as f64 - 1e-9;

    let dw = P::new(w, 0.);
    let dh = P::new(0., h);

    let mut cut_x = vec![];
    let mut cut_y = vec![];

    for i in 1..nw {
        cut_x.push((inner_box.min + dw * i as f64).x);
    }
    for j in 1..nh {
        cut_y.push((inner_box.min + dh * j as f64).y);
    }

    split_problem_from_cut(problem, cut_x, cut_y, None)
}

pub fn split_problem_from_cut(
    problem: Problem,
    cut_x: Vec<f64>,
    cut_y: Vec<f64>,
    pillar_cands: Option<Vec<P>>,
) -> Vec<ExtProblem> {
    if problem.stage.min.x > 0. || problem.stage.min.y > 0. {
        panic!("Unsupported problem");
    }

    let margin = P::new(5., 5.);
    let inner_box = Box2D::new(problem.stage.min + margin, problem.stage.max - margin);

    let mut xs = vec![inner_box.min.x]
        .into_iter()
        .chain(cut_x)
        .chain(vec![inner_box.max.x])
        .collect::<Vec<_>>();
    let mut ys = vec![inner_box.min.y]
        .into_iter()
        .chain(cut_y)
        .chain(vec![inner_box.max.y])
        .collect::<Vec<_>>();

    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let (xs, ys) = (xs, ys);

    let mut mini_problems = vec![];

    let is_right = |i: usize| i + 1 == xs.len();
    let is_top = |j: usize| j + 1 == ys.len();

    for i in 0..xs.len() - 1 {
        for j in 0..ys.len() - 1 {
            if !is_right(i + 1) && !is_top(j + 1) {
                continue;
            }

            let ll = P::new(xs[i], ys[j]).to_point();
            let ur = P::new(xs[i + 1], ys[j + 1]).to_point();

            let mini_stage = Box2D::new(ll - margin, ur + margin);

            let ds = [(0, 0), (1, 0), (1, 1), (0, 1)];

            let mut walls = vec![];
            let mut extra_pillars = vec![];

            for k in 0..4 {
                let (di1, dj1) = ds[k];
                let (di2, dj2) = ds[(k + 1) % 4];

                let (i1, j1) = (i + di1, j + dj1);
                let (i2, j2) = (i + di2, j + dj2);

                if is_right(i1) && is_right(i2) || is_top(j1) && is_top(j2) {
                    continue;
                }

                let mut p1 = P::new(xs[i1], ys[j1]).to_point();
                let mut p2 = P::new(xs[i2], ys[j2]).to_point();

                let pillar_dir = rotate90(p2 - p1).normalize() * -5.;

                let mut limit_point = None;

                if is_right(i1) || is_top(j1) {
                    p1 += (p2 - p1).normalize() * 5.;

                    limit_point = p1.into();

                    if pillar_cands.is_none() {
                        extra_pillars.push(Pillar {
                            center: p1 + pillar_dir,
                            radius: 5.,
                        });
                    }
                }
                if is_right(i2) || is_top(j2) {
                    p2 += (p1 - p2).normalize() * 5.;

                    limit_point = p2.into();

                    if pillar_cands.is_none() {
                        extra_pillars.push(Pillar {
                            center: p2 + pillar_dir,
                            radius: 5.,
                        });
                    }
                }

                let wall = LineSegment { from: p1, to: p2 };

                walls.push(wall);

                // Add from pillar_cands
                let Some(pillar_cands) = &pillar_cands else {continue};

                for p in pillar_cands.iter() {
                    let pillar = Pillar {
                        center: p.to_point(),
                        radius: 5.,
                    };

                    if wall.distance_to_point(p.to_point()) < 5. {
                        extra_pillars.push(pillar);
                        continue;
                    }

                    if mini_stage.contains(p.to_point()) {
                        continue;
                    }

                    let Some(limit_point) = limit_point else {continue};

                    if limit_point.distance_to(p.to_point()) < 10.0 {
                        extra_pillars.push(pillar);
                        continue;
                    }
                }
            }

            assert!(0 < walls.len() && walls.len() < 4);

            if pillar_cands.is_none() {
                assert_eq!(extra_pillars.len(), 2);
            }

            mini_problems.push(ExtProblem {
                problem: Problem {
                    stage: mini_stage,
                    ..problem.clone()
                },
                walls,
                extra_pillars,
                is_right: is_right(i + 1),
                is_top: is_top(j + 1),
            });
        }
    }

    mini_problems
}
