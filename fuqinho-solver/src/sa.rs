use common::board::Board;
use common::{Problem, RawSolution, Solution};
use euclid::default::{Point2D, Vector2D};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use std::path::PathBuf;
use thousands::Separable;

// How to run:
// cargo run --release --bin fuqinho-solver -- --sa --initial-temp=1000000 --iterations=100000000 --problem-id=1

const SOLVER_NAME: &str = "fuqinho-SA";

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum CoolingSchedule {
    Linear,
    Quadratic,
    Exponential,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum AcceptFunction {
    Linear,
    Exponential,
}

pub struct SAConfig {
    pub num_iterations: usize,
    pub initial_temperature: f64,
    pub final_temperature: f64,
    pub solutions_dir: PathBuf,
    pub cooling_schedule: CoolingSchedule,
    pub accept_function: AcceptFunction,
}

fn current_temperature(progress: f64, config: &SAConfig) -> f64 {
    match config.cooling_schedule {
        CoolingSchedule::Linear => (1. - progress) * config.initial_temperature,
        CoolingSchedule::Quadratic => (1. - progress).powi(2) * config.initial_temperature,
        CoolingSchedule::Exponential => {
            config.initial_temperature.powf(1. - progress) * config.final_temperature.powf(progress)
        }
    }
}

fn should_accept(
    cur_score: f64,
    next_score: f64,
    temperature: f64,
    rng: &mut ThreadRng,
    config: &SAConfig,
) -> bool {
    if next_score >= cur_score {
        return true;
    }
    match config.accept_function {
        AcceptFunction::Linear => rng.gen_range(0.0..1.0) * temperature > -(next_score - cur_score),
        AcceptFunction::Exponential => {
            rng.gen_bool(f64::exp((next_score - cur_score) / (temperature + 1e-9)))
        }
    }
}

fn place_musician_randomly(board: &mut Board, m: usize, rng: &mut ThreadRng) {
    loop {
        let x = rng.gen_range(board.prob.stage.min.x..board.prob.stage.max.x);
        let y = rng.gen_range(board.prob.stage.min.y..board.prob.stage.max.y);
        let pos = Point2D::new(x, y);
        if board.can_place(m, pos) {
            board.try_place(m, pos).unwrap();
            break;
        }
    }
}
fn place_musicians_randomly(board: &mut Board, rng: &mut ThreadRng) {
    for m in 0..board.prob.musicians.len() {
        place_musician_randomly(board, m, rng);
        board.set_volume(m, 10.);
    }
}

fn swap_two_musicians(board: &mut Board, m1: usize, m2: usize) {
    if m1 == m2 {
        return;
    }
    let pos1 = board.musicians()[m1].unwrap().0.to_point();
    let pos2 = board.musicians()[m2].unwrap().0.to_point();
    board.unplace(m1);
    board.unplace(m2);
    board.try_place(m1, pos2).unwrap();
    board.try_place(m2, pos1).unwrap();
}

fn move_at_random_pos(board: &mut Board, m: usize, rng: &mut ThreadRng) -> bool {
    let x_min = board.prob.stage.min.x + 10.;
    let x_max = board.prob.stage.max.x - 10.;
    let y_min = board.prob.stage.min.y + 10.;
    let y_max = board.prob.stage.max.y - 10.;
    board.unplace(m);
    loop {
        let x = rng.gen_range(x_min..x_max);
        let y = rng.gen_range(y_min..y_max);
        let pos = Point2D::new(x, y);
        if board.can_place(m, pos) {
            board.try_place(m, pos).unwrap();
            break;
        }
    }
    true
}

fn gradient(board: &Board, m: usize) -> Vector2D<f64> {
    let mut gradient = Vector2D::new(0., 0.);
    let m_pos = board.musicians()[m].unwrap().0;
    for a in 0..board.prob.attendees.len() {
        if board.is_musician_seeing(m, a) {
            let a_pos = board.prob.attendees[a].position;
            let dist2 = (a_pos.x - m_pos.x).powi(2) + (a_pos.y - m_pos.y).powi(2);
            let dq = (a_pos - m_pos).to_vector() * 2. / dist2.powi(2);
            gradient += dq * 1000000. * board.prob.attendees[a].tastes[board.prob.musicians[m]];
        }
    }
    gradient * board.q(m)
}

fn move_at_gradient_direction(board: &mut Board, m: usize, rng: &mut ThreadRng) -> bool {
    let m_pos = board.musicians()[m].unwrap().0;
    let gradient = gradient(board, m);
    if gradient.square_length() < 0.0001 {
        return false;
    }
    let scale: f64 = rng.gen_range(0.0..1.0);
    let dist = 40.0 * scale.powi(2);
    let new_pos = (m_pos + gradient.normalize() * dist).to_point();
    board.unplace(m);
    if board.can_place(m, new_pos) {
        board.try_place(m, new_pos).unwrap();
        true
    } else {
        board.try_place(m, m_pos.to_point()).unwrap();
        false
    }
}

fn collide_at_random_direction(board: &mut Board, m: usize, rng: &mut ThreadRng) -> bool {
    let pos = board.musicians()[m].unwrap().0;

    let angle = rng.gen_range(0.0..2.0) * std::f64::consts::PI;
    let scale: f64 = rng.gen_range(0.0..1.0);
    let dist = 40.0 * scale.powi(2);
    let dx = angle.cos();
    let dy = angle.sin();
    let mut lo = 0.;
    let mut hi = dist;
    board.unplace(m);
    while hi - lo > 0.001 {
        let mi = (lo + hi) / 2.;
        let new_pos = Point2D::new(pos.x + mi * dx, pos.y + mi * dy);
        if board.can_place(m, new_pos) {
            lo = mi;
        } else {
            hi = mi;
        }
    }
    let new_pos = Point2D::new(pos.x + lo * dx, pos.y + lo * dy);
    board.try_place(m, new_pos).unwrap();
    true
}

fn move_at_random_direction(board: &mut Board, m: usize, rng: &mut ThreadRng) -> bool {
    let pos = board.musicians()[m].unwrap().0;

    board.unplace(m);
    let dist = 40.0 * (rng.gen_range(0.0..1.0) as f64).powf(2.);
    let angle = rng.gen_range(0.0..2.0) * std::f64::consts::PI;

    let new_pos = Point2D::new(pos.x + dist * angle.cos(), pos.y + dist * angle.sin());
    if board.can_place(m, new_pos) {
        board.try_place(m, new_pos).unwrap();
        true
    } else {
        board.try_place(m, pos.to_point()).unwrap();
        false
    }
}

pub fn solve_sa(problem: &Problem, problem_id: u32, config: &SAConfig) -> Solution {
    let mut rng = thread_rng();

    let mut board = Board::new(problem_id, problem.clone(), SOLVER_NAME.to_owned(), false);
    place_musicians_randomly(&mut board, &mut rng);
    let mut best_score = board.score_ignore_negative();

    let mut iteration = 0;
    loop {
        iteration += 1;
        let temperature =
            current_temperature(iteration as f64 / config.num_iterations as f64, config);

        if rng.gen_range(0..10) == 0 {
            // 10%: swap two musicians

            let m1 = rng.gen_range(0..board.prob.musicians.len());
            let m2 = rng.gen_range(0..board.prob.musicians.len());
            swap_two_musicians(&mut board, m1, m2);
            let score = board.score_ignore_negative();
            if should_accept(best_score, score, temperature, &mut rng, config) {
                best_score = score;
            } else {
                swap_two_musicians(&mut board, m1, m2);
            }
        } else {
            let r = rng.gen_range(0..10);
            let m = rng.gen_range(0..board.prob.musicians.len());
            let prev_pos = board.musicians()[m].unwrap().0.to_point();
            let moved;
            if r == 0 {
                // 90*10 = 9%: Move a musician at random position.
                moved = move_at_random_pos(&mut board, m, &mut rng);
            } else if r == 1 {
                // 90*10 = 9%: Move random direction for 40 units at max.
                moved = collide_at_random_direction(&mut board, m, &mut rng);
            } else if r == 2 {
                // 90*10 = 9%: Move at gradient direction for 40 units at max.
                moved = move_at_gradient_direction(&mut board, m, &mut rng);
            } else {
                // 90*70 = 63%: Move a musician at random direction for up to 40 units.
                moved = move_at_random_direction(&mut board, m, &mut rng);
            }

            if moved {
                let score = board.score_ignore_negative();
                if should_accept(best_score, score, temperature, &mut rng, config) {
                    best_score = score;
                } else {
                    board.unplace(m);
                    board.try_place(m, prev_pos).unwrap();
                }
            }
        }

        if iteration % 10000 == 0 {
            eprintln!(
                "I:{} T:{:.0} Score:{}",
                iteration,
                temperature,
                best_score.separate_with_commas()
            );
        }
        if iteration % 1000000 == 0 {
            // Write the solution to file.
            let solution_to_write = board.clone().solution().unwrap();
            let solution_json =
                serde_json::to_string(&RawSolution::from(solution_to_write.clone())).unwrap();
            let mut output = config.solutions_dir.clone();
            if !output.is_dir() {
                std::fs::create_dir_all(&output).unwrap();
            }
            output.push(format!(
                "{}-{}M-{}.json",
                solution_to_write.problem_id,
                iteration / 1000000,
                best_score
            ));
            std::fs::write(output, solution_json).unwrap();
        }
        if iteration >= config.num_iterations {
            break;
        }
    }

    board.solution().unwrap()
}

// ======================================================
// 100,000,000 iterations result (problem 1)
// 1) Steps: 10% swap + 9% random pos + 9% random collide + 9% grad dir + 63% random dir
//    Setup: T=1,000,000, linear cooling, linear accept function
//    Score: 16,488,264,680
// 2) Steps: 14% swap + 12% random pos + 12% random collide + 62% random dir
//    Setup: T=1,000,000, linear cooling, linear accept function
//    Score: 16,351,443,820
