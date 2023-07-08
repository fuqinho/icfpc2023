use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
    str::FromStr,
};

use anyhow::{bail, Context};
use common::{board::Board, Problem};
use lyon_geom::{Point, Vector};
use pathfinding::prelude::{kuhn_munkres, Matrix};

pub struct Solver {
    orig_problem: Problem,
    board: Board,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Normal,
    ZigZag,
}

impl FromStr for Algorithm {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Self::Normal),
            "zigzag" => Ok(Self::ZigZag),
            _ => bail!("unknown algorithm name {}", s),
        }
    }
}

type P = Point<f64>;

const D: usize = 10;

impl Solver {
    pub fn new(problem_id: u32, problem: Problem) -> Self {
        let board = Board::new(problem_id, problem.clone());

        Self {
            orig_problem: problem,
            board,
        }
    }

    pub fn solve(&mut self, algo: Algorithm) -> (f64, Board) {
        let mut outer = self.compute_outer(algo);

        let mut max_x: f64 = 0.;
        let mut max_y: f64 = 0.;
        for p in outer.iter() {
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        let bb = self.board.prob.stage;

        for p in outer.iter_mut() {
            p.x += bb.max.x - max_x;
            p.y += bb.max.y - max_y;

            if p.x > bb.max.x {
                p.x = bb.max.x
            }
            if p.y > bb.max.y {
                p.y = bb.max.y
            }
        }

        // Compute scores for outer points
        let mut scores = vec![vec![]; outer.len()];

        {
            let mut prob2 = self.orig_problem.clone();
            let num_instruments = prob2.attendees[0].tastes.len();
            prob2.musicians = vec![];
            for i in 0..num_instruments {
                prob2.musicians.push(i);
            }
            for _ in 0..outer.len() {
                prob2.musicians.push(0);
            }
            let mut board2 = Board::new(0, prob2);

            for (i, o) in outer.iter().enumerate() {
                board2.try_place(num_instruments + i, o.clone()).unwrap();
            }
            for (i, o) in outer.iter().enumerate() {
                board2.unplace(num_instruments + i);

                for j in 0..num_instruments {
                    board2.try_place(j, o.clone()).unwrap();

                    scores[i].push(board2.contribution(j));

                    board2.unplace(j);
                }

                board2.try_place(num_instruments + i, *o).unwrap();
            }
        }

        let mut weights = vec![vec![]; outer.len()];
        for i in 0..outer.len() {
            for j in self.board.prob.musicians.iter() {
                weights[i].push(scores[i][*j] as i64);
            }
        }

        eprintln!(
            "Computing hungarian on {} x {} matrix",
            weights.len(),
            weights[0].len()
        );

        let matrix = Matrix::from_rows(weights.clone()).unwrap();

        let mut to_place: BTreeSet<usize> = (0..self.board.prob.musicians.len()).collect();

        if weights.len() <= weights[0].len() {
            let (_score, assignments) = kuhn_munkres(&matrix);

            for (i, j) in assignments.iter().enumerate() {
                self.board
                    .try_place(*j, outer[i])
                    .context("failed to place")
                    .unwrap();

                to_place.remove(j);
            }
        } else {
            // musician -> circle -> weight
            let (_score, assignments) = kuhn_munkres(&matrix.transposed());

            for (j, i) in assignments.iter().enumerate() {
                self.board
                    .try_place(j, outer[*i])
                    .context("failed to place")
                    .unwrap();

                to_place.remove(&j);
            }
        }

        for x in ((bb.min.x.ceil() as usize)..(bb.max.x.floor() as usize - D)).step_by(D) {
            if to_place.is_empty() {
                break;
            }

            for y in ((bb.min.y).ceil() as usize..bb.max.y.floor() as usize - D).step_by(D) {
                if to_place.is_empty() {
                    break;
                }

                if x == bb.min.x.ceil() as usize || y == bb.min.y.ceil() as usize {
                    continue;
                }

                let i = to_place.first().unwrap();

                if self
                    .board
                    .try_place(*i, Point::new(x as f64, y as f64))
                    .is_ok()
                {
                    to_place.pop_first().unwrap();
                }
            }
        }

        assert!(to_place.is_empty());
        // assert_eq!(self.board.score(), score as f64);

        return (self.board.score() as f64, self.board.clone());
    }

    fn compute_outer(&self, algo: Algorithm) -> Vec<P> {
        let mut outer = vec![];

        let bb = self.board.prob.stage;

        if algo == Algorithm::Normal {
            for x in ((bb.min.x.ceil() as usize)..(bb.max.x.floor() as usize)).step_by(D) {
                if bb.min.y > D as f64 {
                    outer.push(Point::new(x as f64, bb.min.y));
                }
                outer.push(Point::new(x as f64, bb.max.y));
            }

            for y in ((bb.min.y).ceil() as usize + D..bb.max.y.floor() as usize - D).step_by(D) {
                if bb.min.x > D as f64 {
                    outer.push(Point::new(bb.min.x, y as f64));
                }
                outer.push(Point::new(bb.max.x, y as f64));
            }
        } else if algo == Algorithm::ZigZag {
            let mut queue = vec![];
            let mut visited = HashSet::<Point<i64>>::new();

            let mul = 1_000_000i64;
            let sqrt2_5 = 7_071_068i64;

            let init = Point::new(bb.min.x.ceil() as i64 * mul, bb.min.y.ceil() as i64 * mul);
            queue.push(init);
            visited.insert(init);

            let eps = Vector::new(1e-9, 1e-9);
            let mut bb_outer = bb;
            bb_outer.max += eps;

            let mut bb_inner = bb;
            bb_inner.min +=
                Vector::new(sqrt2_5 as f64 / mul as f64, sqrt2_5 as f64 / mul as f64) + eps;
            bb_inner.max -=
                Vector::new(sqrt2_5 as f64 / mul as f64, sqrt2_5 as f64 / mul as f64) * 2. + eps;

            while let Some(p) = queue.pop() {
                for dx in [-1, 1] {
                    for dy in [-1, 1] {
                        let np = p + Vector::new(dx * sqrt2_5, dy * sqrt2_5);

                        if visited.contains(&np) {
                            continue;
                        }

                        let real_np = np.to_f64() / mul as f64;

                        if bb_outer.contains(real_np) && !bb_inner.contains(real_np) {
                            queue.push(np);
                            visited.insert(np);
                        }
                    }
                }
            }

            outer = visited
                .into_iter()
                .map(|p| p.to_f64() / mul as f64)
                .collect();

            outer.sort_by(|x, y| {
                let o = x.x.partial_cmp(&y.x).unwrap();
                if o == Ordering::Equal {
                    x.y.partial_cmp(&y.y).unwrap()
                } else {
                    o
                }
            });
        }

        outer
    }
}
