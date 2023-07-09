use common::{board::Board, Problem, RawSolution, Solution};
use euclid::{default::*, point2, vec2};
use lyon_geom::Point;
use rand::Rng;

const SOLVER_NAME: &str = "manual-annealer";

struct Solver2<'a> {
    problem_id: u32,
    problem: &'a Problem,
    temp: f64,
    initial_solution: &'a common::Solution,
}

#[derive(Clone)]
struct State2 {
    board: Board,
}

enum Move {
    Change {
        id: usize,
        new_pos: Point2D<f64>,
        old_pos: Point2D<f64>,
    },
    Swap {
        i: usize,
        j: usize,
    },
    Multiple {
        moves: Vec<Move>,
    },
}

impl Move {
    fn gen_change(rng: &mut impl Rng, board: &Board, progress_ratio: f64) -> Self {
        let stage = &board.prob.stage;

        let scale_x = (stage.width() / 5.0 * (1.0 - progress_ratio)).max(5.0);
        let scale_y = (stage.height() / 5.0 * (1.0 - progress_ratio)).max(5.0);

        let grid = 0.25_f64;
        let scale_x = (scale_x / grid).round() as i32;
        let scale_y = (scale_y / grid).round() as i32;

        loop {
            let id = rng.gen_range(0..board.musicians().len());

            // let theta = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
            // let len = rng.gen_range(0.1..=scale);
            // let d = Vector2D::from_angle_and_length(Angle::radians(theta), len);

            let d = vec2(
                rng.gen_range(-scale_x..=scale_x) as f64 * grid,
                rng.gen_range(-scale_y..=scale_y) as f64 * grid,
            );

            let old_pos = board.musicians()[id].unwrap().0.to_point();
            let new_pos = old_pos + d;
            let new_pos = point2(
                new_pos.x.clamp(stage.min.x, stage.max.x),
                new_pos.y.clamp(stage.min.y, stage.max.y),
            );

            if new_pos == old_pos {
                continue;
            }

            if !board.can_place(id, new_pos) {
                continue;
            }

            break Move::Change {
                id,
                new_pos,
                old_pos,
            };
        }
    }

    fn gen_swap(rng: &mut impl Rng, board: &Board) -> Self {
        loop {
            let i = rng.gen_range(0..board.prob.musicians.len());
            let j = rng.gen_range(0..board.prob.musicians.len());
            if i != j && board.prob.musicians[i] != board.prob.musicians[j] {
                break Move::Swap { i, j };
            }
        }
    }
}

impl saru::Annealer for Solver2<'_> {
    type State = State2;

    type Move = Move;

    fn init_state(&self, rng: &mut impl Rng) -> Self::State {
        let mut board = Board::new(self.problem_id, self.problem.clone(), SOLVER_NAME);

        for (i, p) in self.initial_solution.placements.iter().enumerate() {
            board.try_place(i, p.position).unwrap();
        }

        State2 { board }
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        self.temp
    }

    fn eval(
        &self,
        state: &Self::State,
        _progress_ratio: f64,
        _best_score: f64,
        _valid_best_score: f64,
    ) -> (f64, Option<f64>) {
        let score = -state.board.score();
        (score, Some(score))
    }

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl Rng,
        progress_ratio: f64,
    ) -> Self::Move {
        match rng.gen_range(0..=4) {
            0..=2 => Move::gen_change(rng, &state.board, progress_ratio),

            3 => loop {
                let m1 = Move::gen_change(rng, &state.board, progress_ratio);
                let m2 = Move::gen_change(rng, &state.board, progress_ratio);

                match (&m1, &m2) {
                    (
                        Move::Change {
                            id: id1,
                            new_pos: new_pos1,
                            ..
                        },
                        Move::Change {
                            id: id2,
                            new_pos: new_pos2,
                            ..
                        },
                    ) => {
                        if id1 == id2 {
                            continue;
                        }
                        if new_pos1.distance_to(*new_pos2) < 10.0 {
                            continue;
                        }
                    }
                    _ => unreachable!(),
                }

                break Move::Multiple {
                    moves: vec![m1, m2],
                };
            },

            4 => Move::gen_swap(rng, &state.board),
            _ => unreachable!(),
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        match mov {
            Move::Change { id, new_pos, .. } => {
                state.board.unplace(*id);
                state.board.try_place(*id, *new_pos).unwrap();
            }
            Move::Swap { i, j } => {
                let pi = state.board.musicians()[*i].unwrap().0.to_point();
                let pj = state.board.musicians()[*j].unwrap().0.to_point();
                state.board.unplace(*i);
                state.board.unplace(*j);
                state.board.try_place(*i, pj).unwrap();
                state.board.try_place(*j, pi).unwrap();
            }
            Move::Multiple { moves } => {
                for mov in moves {
                    self.apply(state, mov);
                }
            }
        }
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        match mov {
            Move::Change { id, old_pos, .. } => {
                state.board.unplace(*id);
                state.board.try_place(*id, *old_pos).unwrap();
            }
            Move::Swap { .. } => {
                self.apply(state, mov);
            }
            Move::Multiple { moves } => {
                for mov in moves.iter().rev() {
                    self.unapply(state, mov);
                }
            }
        }
    }
}

pub fn perform_annealing(
    problem: &Problem,
    initial_solution: &Solution,
    temp: f64,
    time_limit: f64,
    seed: u64,
) -> Solution {
    let solver = Solver2 {
        problem_id: initial_solution.problem_id,
        problem,
        temp,
        initial_solution,
    };

    let solution = saru::annealing(
        &solver,
        &saru::AnnealingOptions {
            time_limit,
            limit_temp: temp,
            restart: 0,
            threads: 1,
            silent: false,
            header: String::new(),
        },
        seed,
    );

    let state = solution.state.expect("Valid solution not found");
    let solution: Solution = state.board.try_into().expect("Failed to finalize Board");

    solution
}
