use std::time::Instant;

use common::{board::Board, Problem};
use lyon_geom::Point;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct Solver {
    board: Board,

    pub best_score: f64,
    pub best_board: Board,

    timeout_secs: u64,

    rng: StdRng,
}

impl Solver {
    pub fn new(problem_id: u32, problem: Problem, timeout_secs: u64, rng_seed: u64) -> Self {
        let board = Board::new(problem_id, problem);

        let rng = StdRng::seed_from_u64(rng_seed);

        Self {
            best_score: f64::NEG_INFINITY,
            best_board: board.clone(),

            board,
            timeout_secs,
            rng,
        }
    }

    fn initialize(&mut self) {
        for i in 0..self.board.prob.musicians.len() {
            self.place_randomly(i);
        }

        self.update_best();
    }

    fn place_randomly(&mut self, i: usize) {
        loop {
            let x: f64 = self
                .rng
                .gen_range(self.board.prob.stage.min.x..self.board.prob.stage.max.x);
            let y: f64 = self
                .rng
                .gen_range(self.board.prob.stage.min.y..self.board.prob.stage.max.y);
            if self.board.try_place(i, Point::new(x, y)).is_ok() {
                break;
            }
        }
    }

    fn update_best(&mut self) {
        if self.best_score < self.board.score() {
            eprintln!("update best score: {}", self.board.score());
            self.best_score = self.board.score();
            self.best_board = self.board.clone();
        }
    }

    pub fn solve(mut self) -> (f64, Board) {
        let instant = Instant::now();

        self.initialize();

        while instant.elapsed().as_secs() < self.timeout_secs {
            for _ in 0..10 {
                let i = self.rng.gen_range(0..self.board.prob.musicians.len());

                for _ in 0..100 {
                    let prev_score = self.board.score();
                    let prev_pos = self.board.musicians()[i].unwrap().0.to_point();

                    self.board.unplace(i);
                    self.place_randomly(i);

                    if prev_score < self.board.score() {
                        self.update_best();
                    } else {
                        self.board.unplace(i);
                        self.board.try_place(i, prev_pos).unwrap();
                    }
                }
            }
        }

        (self.best_score, self.best_board)
    }
}
