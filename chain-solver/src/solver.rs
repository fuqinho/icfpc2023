use std::{collections::BTreeSet, time::Instant};

use anyhow::Context;
use common::{board::Board, geom::rotate90, Problem};
use lyon_geom::Point;
use pathfinding::prelude::{kuhn_munkres, Matrix};
use rand::{rngs::StdRng, Rng, SeedableRng};

const SOLVER_NAME: &str = "chain-solver";

pub struct Solver {
    orig_problem: Problem,
    board: Board,

    // musician -> adjacent musicians
    graph: Vec<BTreeSet<usize>>,

    // ins -> remaining musicians
    remaining: Vec<BTreeSet<usize>>,

    rng: StdRng,
}

type P = Point<f64>;

const D: f64 = 10.;

impl Solver {
    pub fn new(problem_id: u32, problem: Problem, seed: u64) -> Self {
        let board = Board::new(problem_id, problem.clone(), SOLVER_NAME);

        let k = board.prob.attendees[0].tastes.len();

        Self {
            orig_problem: problem,
            graph: vec![BTreeSet::new(); board.prob.musicians.len()],
            board,
            remaining: vec![BTreeSet::new(); k],
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn try_place(&mut self, m: usize, position: Point<f64>) -> anyhow::Result<()> {
        self.board
            .try_place(m, position)
            .context("failed to place")?;

        let ins = self.board.prob.musicians[m];

        self.remaining[ins].remove(&m);

        Ok(())
    }

    fn unplace(&mut self, m: usize) {
        self.board.unplace(m);

        let ins = self.board.prob.musicians[m];

        self.remaining[ins].insert(m);
    }

    pub fn solve(&mut self, hill_climb_ts_secs: u64) -> (f64, Board) {
        let outer = self.compute_outer();

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
            let mut board2 = Board::new(0, prob2, SOLVER_NAME);

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

        if weights.len() <= weights[0].len() {
            let (_score, assignments) = kuhn_munkres(&matrix);

            for (i, j) in assignments.iter().enumerate() {
                self.try_place(*j, outer[i])
                    .context("failed to place")
                    .unwrap();
            }
        } else {
            // musician -> circle -> weight
            let (_score, assignments) = kuhn_munkres(&matrix.transposed());

            for (j, i) in assignments.iter().enumerate() {
                self.try_place(j, outer[*i])
                    .context("failed to place")
                    .unwrap();
            }
        }

        // Update graph
        for i in 0..self.board.prob.musicians.len() {
            for j in 0..self.board.prob.musicians.len() {
                if i == j {
                    continue;
                }
                if let Some(p) = self.board.musicians()[i] {
                    if let Some(q) = self.board.musicians()[j] {
                        if (p.0 - q.0).length() <= D + 1e-9 {
                            self.graph[i].insert(j);
                            self.graph[j].insert(i);
                        }
                    }
                }
            }
        }
        // Update remaining
        for (m, p) in self.board.musicians().iter().enumerate() {
            if p.is_some() {
                continue;
            }
            let ins = self.board.prob.musicians[m];
            self.remaining[ins].insert(m);
        }

        eprintln!("Start hill climbing");

        // Hill climb
        let instant = Instant::now();
        while instant.elapsed().as_secs() < hill_climb_ts_secs
            && self.remaining_misicians_count() > 3
        {
            let mut cand_ids = vec![];
            for (i, p) in self.board.musicians().iter().enumerate() {
                if p.is_some() {
                    cand_ids.push(i);
                }
            }

            let i = self.rng.gen_range(0..cand_ids.len());

            let m0 = cand_ids[i];
            let p0 = self.board.musicians()[m0].unwrap();

            if self.graph[m0].len() < 2 {
                continue;
            }
            assert_eq!(self.graph[m0].len(), 2);

            let nei = self.graph[m0].iter().collect::<Vec<_>>();
            let m1 = *nei[0];
            let m2 = *nei[1];

            let p1 = self.board.musicians()[m1].unwrap().0;
            let p2 = self.board.musicians()[m2].unwrap().0;

            let mid = (p1 + p2) / 2.;

            let r = D / 2. + 1e-9;
            let proj1 = mid + (p1 - mid).normalize() * r;
            let proj2 = mid + (p2 - mid).normalize() * r;

            let d = (p1 - proj1).length();
            let l = ((4. * r * r) - d * d).sqrt();

            let n = rotate90(p2 - p1).normalize();

            let mut nps = vec![];

            for sig in [-1., 1.] {
                nps.push((proj1 + n * sig * l, proj2 + n * sig * l));
            }

            let prev_score = self.board.score();

            self.unplace(m0);

            let mut found = false;
            for (q1, q2) in nps {
                // Place random two remaining musicians to p1 and p2.
                let repr1 = self.random_remaining_musician().unwrap();
                let repr2 = self.random_remaining_musician().unwrap();
                if repr1 == repr2 {
                    continue;
                }

                if self.try_place(repr1, q1.to_point()).is_err() {
                    continue;
                }
                if self.try_place(repr2, q2.to_point()).is_err() {
                    self.unplace(repr1);
                    continue;
                }

                // Choose cands from the remaining musicians.
                let mut cands = vec![];

                for (_, rem) in self.remaining.iter().clone().enumerate() {
                    if rem.is_empty() {
                        continue;
                    }
                    let f = *rem.first().unwrap();
                    let l = *rem.last().unwrap();

                    cands.push(f);
                    if f != l {
                        cands.push(l);
                    }
                }

                let mut weights = vec![vec![]; 2];
                for i in 0..2 {
                    for j in cands.iter() {
                        let ins = self.board.prob.musicians[*j];

                        weights[i].push(
                            self.board
                                .contribution_if_instrument([repr1, repr2][i], ins)
                                as i64,
                        );
                    }
                }

                let matrix = Matrix::from_rows(weights.clone()).unwrap();

                let assignments = kuhn_munkres(&matrix);

                for i in 0..2 {
                    self.unplace([repr1, repr2][i]);

                    self.try_place(cands[assignments.1[i]], [q1, q2][i].to_point())
                        .unwrap();
                }

                let cur_score = self.board.score();

                if cur_score > prev_score {
                    found = true;

                    eprintln!("Score improved from {} to {}", prev_score, cur_score);

                    // Update graph
                    self.graph[m0].remove(&m1);
                    self.graph[m0].remove(&m2);
                    self.graph[m1].remove(&m0);
                    self.graph[m2].remove(&m0);

                    let n1 = cands[assignments.1[0]];
                    let n2 = cands[assignments.1[1]];
                    for e in [(m1, n1), (n1, n2), (n2, m2)] {
                        self.graph[e.0].insert(e.1);
                        self.graph[e.1].insert(e.0);
                    }

                    break;
                }

                // Undo
                self.unplace(cands[assignments.1[0]]);
                self.unplace(cands[assignments.1[1]]);
            }

            if !found {
                self.try_place(m0, p0.0.to_point()).unwrap();
            }
        }

        // Place remaining points
        let bb = self.board.prob.stage;
        for x in ((bb.min.x.ceil() as usize)..(bb.max.x.floor() - D) as usize).step_by(D as usize) {
            for y in
                ((bb.min.y).ceil() as usize..(bb.max.y.floor() - D) as usize).step_by(D as usize)
            {
                if self.remaining_misicians_count() == 0 {
                    break;
                }

                if x == bb.min.x.ceil() as usize || y == bb.min.y.ceil() as usize {
                    continue;
                }

                let i = self.random_remaining_musician().unwrap();

                if self.try_place(i, Point::new(x as f64, y as f64)).is_ok() {}
            }
        }

        return (self.board.score() as f64, self.board.clone());
    }

    fn compute_outer(&self) -> Vec<P> {
        let mut outer = vec![];

        let bb = self.board.prob.stage;

        // let from = Point::new(bb.min.x, bb.max.y);
        // let to = Point::new(bb.max.x, bb.min.y);

        // let l = (from - to).length();

        // let dd = D + 1e-9;
        // let num = (l / dd).floor() as usize;

        // for i in 0..num {
        //     let p = from + (to - from).normalize() * dd * i as f64;
        //     outer.push(p);
        // }

        for x in ((bb.min.x.ceil() as usize)..(bb.max.x.floor() as usize)).step_by(D as usize) {
            if bb.min.y > D as f64 {
                outer.push(Point::new(x as f64, bb.min.y));
            }
            outer.push(Point::new(x as f64, bb.max.y));
        }

        for y in ((bb.min.y).ceil() as usize + D as usize..bb.max.y.floor() as usize - D as usize)
            .step_by(D as usize)
        {
            if bb.min.x > D as f64 {
                outer.push(Point::new(bb.min.x, y as f64));
            }
            outer.push(Point::new(bb.max.x, y as f64));
        }

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

        outer
    }

    fn random_remaining_musician(&mut self) -> Option<usize> {
        let mut ids = vec![];
        for (i, rem) in self.remaining.iter().enumerate() {
            if !rem.is_empty() {
                ids.push(i);
            }
        }
        if ids.is_empty() {
            return None;
        }
        let i = self.rng.gen_range(0..ids.len());

        let d: bool = self.rng.gen();

        let rem = &self.remaining[ids[i]];

        if d {
            (*rem.first().unwrap()).into()
        } else {
            (*rem.last().unwrap()).into()
        }
    }

    fn remaining_misicians_count(&self) -> usize {
        let mut res = 0;
        for rem in self.remaining.iter() {
            res += rem.len();
        }
        res
    }
}
