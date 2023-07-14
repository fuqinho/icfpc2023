use std::f64::consts::PI;

use common::{board::Board, board_options::BoardOptions, Problem};
use lyon_geom::Vector;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::pretty::pretty;

pub struct Solver {
    board: Board,
    num_iter: usize,
    rng: SmallRng,
}

type P = Vector<f64>;

impl Solver {
    pub fn new(problem_id: u32, problem: Problem, num_iter: usize) -> Self {
        if problem.stage.min.x > 0. || problem.stage.min.y > 0. {
            panic!("Unsupported stage min: {:?}", problem.stage.min);
        }

        let important_attendees_ratio = 0.2;
        let options =
            BoardOptions::default().with_important_attendees_ratio(important_attendees_ratio);

        let board =
            Board::new_with_options(problem_id, problem, "upsolve-oka-solver", false, options);

        let rng = SmallRng::seed_from_u64(0);

        Self {
            board,
            num_iter,
            rng,
        }
    }

    pub fn solve(&mut self) -> Board {
        // Initialize with random positions
        for i in 0..self.board.musicians().len() {
            self.board.set_volume(i, 10.0);

            loop {
                let p = self.random_place();

                if self.board.try_place(i, p.to_point()).is_ok() {
                    break;
                }
            }
        }

        for iter in 0..self.num_iter {
            self.step(iter);

            if iter % (self.num_iter / 100) == 0 {
                let temp = self.temp(iter);
                eprintln!(
                    "iter: {:>10}  score: {:>14}  temp: {:>10}",
                    pretty(iter as i64),
                    pretty(self.board.score() as i64),
                    pretty(temp as i64),
                );
            }
        }

        self.board.clone()
    }

    fn temp(&self, iter: usize) -> f64 {
        let max_temp = 10_000_000.0;
        let min_temp = 0.0;

        (max_temp * (self.num_iter - iter) as f64 + min_temp * iter as f64) / (self.num_iter as f64)
    }

    fn step(&mut self, iter: usize) {
        let score = self.board.score();

        let action = self.random_action();

        if !self.apply(action) {
            return;
        }

        let improve = self.board.score() - score;

        if self.accept(iter, improve) {
            return;
        }

        self.apply(action.invert());
    }

    fn accept(&mut self, iter: usize, improve: f64) -> bool {
        if improve >= 0. {
            return true;
        }
        let temp = self.temp(iter);
        let accept_prob = (improve / temp).exp();

        self.rng.gen_range(0.0..1.0) < accept_prob
    }

    fn apply(&mut self, action: Action) -> bool {
        match action {
            Action::Swap(x, y) => {
                let xp = self.board.musicians()[x].unwrap().0;
                let yp = self.board.musicians()[y].unwrap().0;

                self.board.unplace(x);
                self.board.unplace(y);

                self.board.try_place(x, yp.to_point()).unwrap();
                self.board.try_place(y, xp.to_point()).unwrap();

                return true;
            }
            Action::MoveRandom(m, _, p) => {
                let orig = self.board.musicians()[m].unwrap().0;

                self.board.unplace(m);

                if self.board.try_place(m, p.to_point()).is_ok() {
                    return true;
                }
                self.board.try_place(m, orig.to_point()).unwrap();
                return false;
            }
            Action::MoveDir(m, dir) => {
                let orig = self.board.musicians()[m].unwrap().0;

                self.board.unplace(m);

                if self.board.try_place(m, (orig + dir).to_point()).is_ok() {
                    return true;
                }
                self.board.try_place(m, orig.to_point()).unwrap();
                return false;
            }
        }
    }

    fn random_action(&mut self) -> Action {
        loop {
            let v = self.rng.gen_range(0..100);

            match v {
                // Swap random two musicianss
                0..=9 => {
                    let x = self.random_musician();
                    let y = self.random_musician();

                    if x == y {
                        continue;
                    }

                    return Action::Swap(x, y);
                }
                // Move a musician to a random place
                10..=19 => {
                    let x = self.random_musician();
                    let orig = self.board.musicians()[x].unwrap().0;
                    let p = self.random_place();

                    return Action::MoveRandom(x, orig, p);
                }
                // Move a musician to a random direction
                20..=99 => {
                    let x = self.random_musician();
                    let dir = self.random_direction();

                    return Action::MoveDir(x, dir);
                }
                _ => continue,
            }
        }
    }

    fn random_direction(&mut self) -> P {
        let d: f64 = self.rng.gen_range(0.0..1.0);
        let r = self.rng.gen_range(-1.0..1.0) * PI;

        let (x, y) = r.sin_cos();

        let dd = d.powi(2);

        P::new(x * 40. * dd, y * 40. * dd)
    }

    fn random_musician(&mut self) -> usize {
        self.rng.gen_range(0..self.board.musicians().len())
    }

    fn random_place(&mut self) -> P {
        let x = self
            .rng
            .gen_range(self.board.prob.stage.min.x..self.board.prob.stage.max.x);
        let y = self
            .rng
            .gen_range(self.board.prob.stage.min.y..self.board.prob.stage.max.y);

        P::new(x, y)
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Swap(usize, usize),
    MoveRandom(usize, /* from */ P, /* to */ P),
    MoveDir(usize, P),
}

impl Action {
    fn invert(&self) -> Action {
        match self {
            Action::Swap(x, y) => Action::Swap(*x, *y),
            Action::MoveRandom(m, orig, p) => Action::MoveRandom(*m, *p, *orig),
            Action::MoveDir(m, p) => Action::MoveDir(*m, -*p),
        }
    }
}
