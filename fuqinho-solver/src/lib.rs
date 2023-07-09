use bitset_fixed::BitSet;
use common::board::Board;
use common::{evaluate, Placement, Problem, Solution};
use euclid::default::Point2D;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::thread;
use thousands::Separable;

const SOLVER_NAME: &str = "fuqinho-solver";
const BEAM_WIDTH: usize = 10;

#[derive(Debug, Clone, Copy)]
pub struct CandidatePos {
    pub pos: Point2D<f64>,
    pub score: f64,
}

struct Config {
    rng: ThreadRng,
    locations: Vec<CandidatePos>,
}

#[allow(dead_code)]
struct State {
    score: i64,
    placements: Vec<u16>,
    placed: BitSet,
    used: BitSet,
    board: Board,
}

#[allow(dead_code)]
struct Move {
    score: i64,
    musician: usize,
    instrument: usize,
    location: usize,
    from: usize,
}

#[allow(dead_code)]
impl State {
    fn pick_random_avail_musician(&self, config: &mut Config) -> usize {
        loop {
            let musician = config.rng.gen_range(0..self.board.prob.musicians.len());
            if !self.placed[musician] {
                return musician;
            }
        }
    }

    fn pick_random_avail_location(&self, config: &mut Config) -> usize {
        loop {
            let loc_id = config.rng.gen_range(0..config.locations.len());
            if self.placements[loc_id] == u16::MAX {
                return loc_id;
            }
        }
    }
}

pub fn generate_line_points(start_x: f64, end_x: f64, y: f64, vertical: bool) -> Vec<CandidatePos> {
    let mut result = vec![];
    let mut x = start_x;
    while x >= start_x.min(end_x) && x <= start_x.max(end_x) {
        result.push(CandidatePos {
            pos: if vertical {
                Point2D::new(y, x)
            } else {
                Point2D::new(x, y)
            },
            score: 0.0,
        });
        if start_x <= end_x {
            x += 10.0;
        } else {
            x -= 10.0;
        }
    }
    result
}

pub fn generate_grid_points(problem: &Problem) -> Vec<CandidatePos> {
    let mut result = vec![];
    for iteration in 0..2 {
        let min_x = problem.stage.min.x + 10.0 + 10.0 * iteration as f64;
        let max_x = problem.stage.max.x - 10.0 - 10.0 * iteration as f64;
        let min_y = problem.stage.min.y + 10.0 + 10.0 * iteration as f64;
        let max_y = problem.stage.max.y - 10.0 - 10.0 * iteration as f64;
        if problem.stage.min.y != 0.0 {
            let sx = min_x;
            let ex = max_x - 10.0;
            let y = min_y;
            result.append(&mut generate_line_points(sx, ex, y, false));
        }
        if problem.stage.max.x != problem.room.max.x {
            let sy = min_y;
            let ey = max_y - 10.0;
            let x = max_x;
            result.append(&mut generate_line_points(sy, ey, x, true));
        }
        if problem.stage.max.y != problem.room.max.y {
            let sx = max_x;
            let ex = min_x + 10.0;
            let y = max_y;
            result.append(&mut generate_line_points(sx, ex, y, false));
        }
        if problem.stage.min.x != 0.0 {
            let sy = max_y;
            let ey = min_y + 10.0;
            let x = min_x;
            result.append(&mut generate_line_points(sy, ey, x, true));
        }
    }

    result
}

pub fn solve_one(problem: &Problem, problem_id: usize) -> Solution {
    // 1. Generate candidate grid points.
    let grid_points = generate_grid_points(problem);

    // 2. Compute score for each points and filter out unimportant points.

    // 3.

    let mut config = Config {
        rng: rand::thread_rng(),
        locations: grid_points,
    };
    let initial_state = State {
        score: 0,
        placed: BitSet::new(problem.musicians.len()),
        used: BitSet::new(config.locations.len()),
        placements: vec![u16::MAX; config.locations.len()],
        board: Board::new(problem_id as u32, problem.clone(), SOLVER_NAME),
    };
    let mut beam = vec![initial_state];

    for mid in 0..problem.musicians.len() {
        eprintln!("Processing musician {}/{}", mid, problem.musicians.len());
        // List up moves and their estimated scores.
        let mut next_moves = vec![];

        for b in 0..beam.len() {
            for _ in 0..5 {
                // Pick a musician to place.
                let m = beam[b].pick_random_avail_musician(&mut config);
                let instrument = problem.musicians[m];

                // Pick a location to place.
                for i in 0..beam[b].placements.len() {
                    if beam[b].placements[i] != u16::MAX {
                        continue;
                    }

                    beam[b].board.try_place(m, config.locations[i].pos).unwrap();
                    let score = -beam[b].board.score() as i64;
                    beam[b].board.unplace(m);

                    next_moves.push(Move {
                        score,
                        musician: m,
                        instrument,
                        location: i,
                        from: b,
                    });
                }
            }
        }
        next_moves.sort_by_key(|m| m.score);

        let mut next_beam = vec![];
        for next_move in next_moves {
            let mut placements = beam[next_move.from].placements.clone();
            let mut placed = beam[next_move.from].placed.clone();
            placed.set(next_move.musician, true);
            placements[next_move.location] = next_move.musician as u16;
            let pos = config.locations[next_move.location].pos;
            let mut board = beam[next_move.from].board.clone();
            board.try_place(next_move.musician, pos).unwrap();
            next_beam.push(State {
                score: -board.score() as i64,
                used: BitSet::new(config.locations.len()),
                placed,
                placements,
                board,
            });

            if next_beam.len() == BEAM_WIDTH {
                break;
            }
        }
        beam = next_beam;
    }

    let mut placements = vec![
        Placement {
            position: Point2D::new(0.0, 0.0),
        };
        problem.musicians.len()
    ];
    for i in 0..beam[0].placements.len() {
        let musician = beam[0].placements[i];

        if musician == u16::MAX {
            continue;
        }
        eprintln!("musician: {}", musician);
        placements[musician as usize].position = config.locations[i].pos;
    }

    Solution {
        problem_id: problem_id as u32,
        solver: SOLVER_NAME.to_owned(),
        placements,
    }
}

pub fn solve(problem: &Problem, problem_id: usize) -> Solution {
    let mut best_score = -1.0;
    let mut best_solution: Option<Solution> = None;

    let mut threads = vec![];

    for _ in 0..1 {
        threads.push(thread::spawn(|| {}));
        let solution = solve_one(problem, problem_id);
        let score = evaluate(&problem, &solution);
        if score > best_score {
            best_score = score;
            best_solution = Some(solution);
        }
        eprintln!("best_score: {}", best_score.separate_with_commas());
    }

    best_solution.unwrap()
}
