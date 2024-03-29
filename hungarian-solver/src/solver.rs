use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
    fmt::Display,
    str::FromStr,
};

use anyhow::{bail, Context};
use common::{
    api::{get_best_solution, Client},
    board::Board,
    geom::tangent_circle,
    Problem, Solution,
};
use lyon_geom::{Point, Vector};
use pathfinding::prelude::{kuhn_munkres, Matrix};
use rand::{thread_rng, Rng};

const SOLVER_NAME: &str = "hungarian-solver";

pub struct Solver {
    algo: Algorithm,

    orig_problem: Problem,
    board: Board,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Normal,
    ZigZag,
    Gap,
    Stdin,
    FetchBest,
}

impl FromStr for Algorithm {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Self::Normal),
            "zigzag" => Ok(Self::ZigZag),
            "gap" => Ok(Self::Gap),
            "stdin" => Ok(Self::Stdin),
            "fetch" => Ok(Self::FetchBest),
            _ => bail!("unknown algorithm name {}", s),
        }
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::ZigZag => write!(f, "zigzag"),
            Self::Gap => write!(f, "gap"),
            Self::Stdin => write!(f, "stdin"),
            Self::FetchBest => write!(f, "fetch"),
        }
    }
}

type P = Point<f64>;

const D: usize = 10;

impl Solver {
    pub fn new(problem_id: u32, problem: Problem, algo: Algorithm) -> Self {
        let board = Board::new(
            problem_id,
            problem.clone(),
            format!("{}-{}", SOLVER_NAME, algo.to_string()),
            false,
        );

        Self {
            algo,
            orig_problem: problem,
            board,
        }
    }

    pub fn solve(&mut self, post_process: bool) -> (f64, Board) {
        let outer: Vec<euclid::Point2D<f64, euclid::UnknownUnit>> = self.compute_outer(self.algo);

        let res1 = self.solve_with_positions(&outer);

        if !post_process {
            return res1;
        }

        if post_process {
            self.post_process();
        }
        (self.board.score(), self.board.clone())
    }

    pub fn solve_with_positions(&mut self, positions: &Vec<P>) -> (f64, Board) {
        let outer = positions.clone();

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
            let mut board2 = Board::new(0, prob2, "none", false);

            for (i, o) in outer.iter().enumerate() {
                board2.try_place(num_instruments + i, o.clone()).unwrap();
            }

            for (i, _) in outer.iter().enumerate() {
                for ins in self.board.prob.musicians.iter() {
                    scores[i].push(board2.contribution_if_instrument(num_instruments + i, *ins));
                }
            }
        }

        let mut weights = vec![vec![]; outer.len()];
        for i in 0..outer.len() {
            for j in self.board.prob.musicians.iter() {
                let mut s = scores[i][*j] as i64;
                if s < 0 {
                    s = 0;
                }

                weights[i].push(s);
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

        let bb = self.board.prob.stage;

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

    fn compute_outer(&mut self, algo: Algorithm) -> Vec<P> {
        let mut outer = vec![];

        let bb = self.board.prob.stage;

        match algo {
            Algorithm::Normal => {
                for x in ((bb.min.x.ceil() as usize)..(bb.max.x.floor() as usize)).step_by(D) {
                    if bb.min.y > D as f64 {
                        outer.push(Point::new(x as f64, bb.min.y));
                    }
                    outer.push(Point::new(x as f64, bb.max.y));
                }

                for y in ((bb.min.y).ceil() as usize + D..bb.max.y.floor() as usize - D).step_by(D)
                {
                    if bb.min.x > D as f64 {
                        outer.push(Point::new(bb.min.x, y as f64));
                    }
                    outer.push(Point::new(bb.max.x, y as f64));
                }
            }
            Algorithm::ZigZag => {
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
                    Vector::new(sqrt2_5 as f64 / mul as f64, sqrt2_5 as f64 / mul as f64) * 2.
                        + eps;

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
            Algorithm::Gap => {
                for x in ((bb.min.x.ceil() as usize)..(bb.max.x.floor() as usize)).step_by(D) {
                    if bb.min.y > D as f64 {
                        outer.push(Point::new(x as f64, bb.min.y));
                    }
                    outer.push(Point::new(x as f64, bb.max.y));
                }

                for y in ((bb.min.y).ceil() as usize + D..bb.max.y.floor() as usize - D).step_by(D)
                {
                    if bb.min.x > D as f64 {
                        outer.push(Point::new(bb.min.x, y as f64));
                    }
                    outer.push(Point::new(bb.max.x, y as f64));
                }

                let mut min_x: f64 = 1e9;
                let mut min_y: f64 = 1e9;
                let mut max_x: f64 = -1e9;
                let mut max_y: f64 = -1e9;

                for p in outer.iter() {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }

                let outmost = outer.clone();

                for o1 in outmost.iter() {
                    for o2 in outmost.iter() {
                        if o1 == o2 {
                            continue;
                        }

                        if o1.x != o2.x && o1.y != o2.y {
                            continue;
                        }

                        if (*o1 - o2.to_vector()).to_vector().square_length() <= (D * D) as f64 {
                            let c = tangent_circle(
                                o1.to_vector(),
                                o2.to_vector(),
                                D as f64 / 2. + 1e-9,
                            )
                            .unwrap();

                            if self.board.prob.stage.contains(c.to_point()) {
                                // check collision
                                let mut ok = true;
                                for p in outer.iter() {
                                    if (*p - c).to_vector().square_length() < (D * D) as f64 {
                                        ok = false;
                                        break;
                                    }
                                }

                                if ok {
                                    outer.push(c.to_point());
                                }
                            }
                        }
                    }
                }
            }
            Algorithm::Stdin => {
                let solution = Solution::read_from_file("/dev/stdin").unwrap();
                outer = solution.placements.iter().map(|p| p.position).collect();
            }
            Algorithm::FetchBest => {
                let solution = get_best_solution(self.board.problem_id).unwrap();

                self.board
                    .solver
                    .push_str(format!("-{}", solution.solver).as_str());

                outer = solution.placements.iter().map(|p| p.position).collect();
            }
        }

        if algo == Algorithm::Gap || algo == Algorithm::Normal || algo == Algorithm::ZigZag {
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
        }

        outer
    }

    fn post_process(&mut self) {
        println!("post processing...");

        let mut rng = thread_rng();

        let mut best_score = self.board.score() as i64;
        for iter in 0.. {
            println!("loop {}", iter);
            let mut improved = false;

            let mut best_remove_pos = 0;
            let mut best_assignments = vec![];

            let current_positions = self
                .board
                .musicians()
                .clone()
                .into_iter()
                .map(|x| x.unwrap())
                .collect::<Vec<_>>();
            let n = current_positions.len();

            for pos in 0..n {
                if self.board.contribution(pos) <= 0. {
                    continue;
                }

                let (p, _) = self.board.musicians()[pos].unwrap();

                self.board.unplace(pos);

                let neg_inf = -1e9 as i64;
                let mut weights = vec![vec![neg_inf; n]; n];

                for pos2 in 0..n {
                    if pos == pos2 {
                        continue;
                    }
                    for m in 0..n {
                        let ins = self.board.prob.musicians[m];

                        let w = self.board.contribution_if_instrument(pos2, ins);

                        weights[pos2][m] = (w as i64).max(0);
                    }
                }

                let matrix = Matrix::from_rows(weights).unwrap();

                // position index -> musician index
                let (score, assignments) = kuhn_munkres(&matrix);
                let score = score - neg_inf;

                if best_score < score {
                    println!("!!! improved score: {} -> {}", best_score, score);
                    improved = true;
                    best_score = score;
                    best_remove_pos = pos;
                    best_assignments = assignments;
                }

                self.board.try_place(pos, p.to_point()).unwrap();
            }

            if !improved {
                break;
            }

            // Remove all
            for i in 0..n {
                self.board.unplace(i);
            }

            for pos in 0..n {
                let m = best_assignments[pos];

                if pos == best_remove_pos {
                    continue;
                }
                self.board
                    .try_place(m, current_positions[pos].0.to_point())
                    .unwrap();
            }

            let m_to_place = best_assignments[best_remove_pos];

            loop {
                let x = rng.gen_range(
                    self.board.prob.stage.min.x as usize..=self.board.prob.stage.max.x as usize,
                );
                let y = rng.gen_range(
                    self.board.prob.stage.min.y as usize..=self.board.prob.stage.max.y as usize,
                );

                if self
                    .board
                    .try_place(m_to_place, Point::new(x as f64, y as f64))
                    .is_ok()
                {
                    // TODO: check score
                    break;
                }
            }

            let solution = self.board.solution_with_optimized_volume().unwrap();

            Client::new()
                .post_submission(self.board.problem_id, solution)
                .unwrap();
        }
    }
}
