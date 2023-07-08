use anyhow::Result;
use common::{api::Client, board::Board, Problem, RawSolution, Solution};
use euclid::{default::*, point2, rect, size2, vec2};
use lyon_geom::{LineSegment, Point};
use rand::Rng;
use std::path::PathBuf;
// use parry2d_f64::{
//     math::{Isometry, Point, Vector},
//     query::{DefaultQueryDispatcher, Ray, RayCast},
//     shape::{Compound, SharedShape},
// };

const PROBLEM_PATH: &str = "../problems";
const EPS: f64 = 1e-6;

struct Solver(Problem);

struct Atendee {
    pos: Point2D<f64>,
    tastes: Vec<f64>,
}

#[derive(Clone)]
struct State {
    placement: Vec<Point2D<f64>>,
    attendee_to_musician: Vec<Vec<AttendeeToMusician>>,
    score: f64,
    impact: Vec<f64>,
}

#[derive(Clone)]
struct AttendeeToMusician {
    line: LineSegment<f64>,
    block_dist: f64,
}

impl State {
    fn new(placement: Vec<Point2D<f64>>, s: &Solver) -> Self {
        let s = &s.0;
        let mut ret = Self {
            placement,
            attendee_to_musician: vec![
                vec![
                    AttendeeToMusician {
                        line: LineSegment {
                            from: point2(0.0, 0.0),
                            to: point2(0.0, 0.0),
                        },
                        block_dist: 0.0
                    };
                    s.musicians.len()
                ];
                s.attendees.len()
            ],
            score: 0.0,
            impact: vec![0.0; s.musicians.len()],
        };

        for i in 0..s.attendees.len() {
            for k in 0..ret.placement.len() {
                let line = LineSegment {
                    from: s.attendees[i].position,
                    to: ret.placement[k],
                };

                let mut block_dist = f64::MAX;
                for l in 0..ret.placement.len() {
                    if l == k {
                        continue;
                    }
                    block_dist = block_dist.min(line.distance_to_point(ret.placement[l]));
                }

                ret.attendee_to_musician[i][k].line = line;
                ret.attendee_to_musician[i][k].block_dist = block_dist;
            }
        }

        ret
    }

    fn change(&mut self, s: &Solver, k: usize, new_pos: Point2D<f64>) {
        let s = &s.0;
        for i in 0..s.attendees.len() {
            let line = LineSegment {
                from: s.attendees[i].position,
                to: new_pos,
            };

            let taste = s.attendees[i].tastes[s.musicians[k]];
            let sc = (1_000_000.0 * taste / line.square_length()).ceil();

            if sc.abs() < 100.0 {
                self.attendee_to_musician[i][k].line = line;
                self.attendee_to_musician[i][k].block_dist = 0.0;
                continue;
            }

            let mut block_dist = f64::MAX;
            for l in 0..self.placement.len() {
                if l == k {
                    continue;
                }
                block_dist = block_dist.min(line.square_distance_to_point(self.placement[l]));
            }

            self.attendee_to_musician[i][k].line = line;
            self.attendee_to_musician[i][k].block_dist = block_dist.sqrt();
        }

        let old_pos = self.placement[k];
        self.placement[k] = new_pos;

        for i in 0..s.attendees.len() {
            for l in 0..self.placement.len() {
                if l == k {
                    continue;
                }

                let line = self.attendee_to_musician[i][l].line;

                let taste = s.attendees[i].tastes[s.musicians[l]];
                let sc = (1_000_000.0 * taste / line.square_length()).ceil();

                if sc.abs() < 100.0 {
                    self.attendee_to_musician[i][l].line = line;
                    self.attendee_to_musician[i][l].block_dist = 0.0;
                    continue;
                }

                let d = line.distance_to_point(old_pos);
                if self.attendee_to_musician[i][l].block_dist != 0.0
                    && self.attendee_to_musician[i][l].block_dist + EPS < d
                {
                    continue;
                }

                // changed minimum dist. recalculate all.
                let mut block_dist = f64::MAX;
                for m in 0..self.placement.len() {
                    if l == m {
                        continue;
                    }
                    block_dist = block_dist.min(line.square_distance_to_point(self.placement[m]));
                }

                self.attendee_to_musician[i][l].block_dist = block_dist.sqrt();
            }
        }
    }

    fn eval_mut(&mut self, s: &Solver) {
        let s = &s.0;
        let mut score = 0.0;

        for k in 0..self.placement.len() {
            self.impact[k] = 0.0;
        }

        for i in 0..s.attendees.len() {
            for k in 0..self.placement.len() {
                if self.attendee_to_musician[i][k].block_dist > 5.0 {
                    let d = self.attendee_to_musician[i][k].line.square_length();
                    let taste = s.attendees[i].tastes[s.musicians[k]];
                    let sc = (1_000_000.0 * taste / d).ceil();
                    score += sc;
                    self.impact[k] += sc.abs();
                }
            }
        }

        self.score = score;
    }

    fn eval(&self) -> (f64, bool) {
        (self.score, true)
    }

    // #[allow(unused)]
    // fn parry_eval(&self, s: &Solver) -> (f64, bool) {
    //     let s = &s.0;
    //     let comp = Compound::new(
    //         self.placement
    //             .iter()
    //             .map(|p| (Isometry::translation(p.x, p.y), SharedShape::ball(5.0)))
    //             .collect(),
    //     );

    //     let mut ret = 0.0;
    //     let qd = DefaultQueryDispatcher;

    //     for attendee in &s.attendees {
    //         let iso = Isometry::translation(0.0, 0.0);
    //         for (k, p) in self.placement.iter().enumerate() {
    //             let dir: Vector2D<f64> = *p - attendee.position;
    //             let dir_norm = dir.normalize();
    //             let dir_len = dir.length();
    //             let ray = Ray::new(
    //                 Point::new(attendee.position.x, attendee.position.y),
    //                 Vector::new(dir_norm.x, dir_norm.y),
    //             );
    //             if !comp.intersects_ray(&iso, &ray, dir_len - (5.0 + EPS)) {
    //                 ret += (1_000_000.0 * attendee.tastes[s.musicians[k]] / dir_len.powi(2)).ceil();
    //             }
    //             // let to = *p + dir_norm * (dir_len - (5.0 + EPS));
    //             // let seg = Segment::new(
    //             //     Point::new(attendee.pos.x, attendee.pos.y),
    //             //     Point::new(to.x, to.y),
    //             // );
    //             // if contact_composite_shape_shape(&qd, &iso, &comp, &seg, 0.0).is_some() {
    //             //     ret += (1_000_000.0 * attendee.tastes[s.musicians[k]] / dir_len.powi(2)).ceil();
    //             // }
    //         }
    //     }

    //     (ret, true)
    // }

    #[allow(unused)]
    fn naive_eval(&self, s: &Solver) -> (f64, bool) {
        let s = &s.0;
        let mut ret = 0.0;

        for i in 0..s.attendees.len() {
            for k in 0..self.placement.len() {
                let line = LineSegment {
                    from: s.attendees[i].position,
                    to: self.placement[k],
                };

                let taste = s.attendees[i].tastes[s.musicians[k]];
                let sc = (1_000_000.0 * taste / line.square_length()).ceil();

                let mut block_dist = f64::MAX;
                for l in 0..self.placement.len() {
                    if l == k {
                        continue;
                    }
                    block_dist = block_dist.min(line.distance_to_point(self.placement[l]));
                }

                if block_dist > 5.0 {
                    let taste = s.attendees[i].tastes[s.musicians[k]];
                    ret += sc;
                }
            }
        }

        (ret, true)
    }
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
}

impl Solver {
    fn stage_valid(&self) -> Box2D<f64> {
        let s = &self.0;
        Box2D::new(
            s.stage.min + vec2(10.0, 10.0),
            s.stage.max - vec2(10.0, 10.0),
        )
    }
}

impl saru::Annealer for Solver {
    type State = State;

    type Move = Move;

    fn init_state(&self, rng: &mut impl Rng) -> Self::State {
        let s = &self.0;
        let stage_valid = self.stage_valid();
        let n = s.musicians.len();

        let mut placement = vec![];

        for _ in 0..n {
            loop {
                let x = rng.gen_range(stage_valid.min.x..=stage_valid.max.x);
                let y = rng.gen_range(stage_valid.min.y..=stage_valid.max.y);

                let p = point2(x, y);
                if placement.iter().any(|q| p.distance_to(*q) < 10.0) {
                    continue;
                }
                placement.push(point2(x, y));
                break;
            }
        }

        let mut ret = State::new(placement, self);
        ret.eval_mut(self);
        ret
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        (init_score.abs() / 10.0).max(1e8)
    }

    fn eval(
        &self,
        state: &Self::State,
        _progress_ratio: f64,
        _best_score: f64,
        _valid_best_score: f64,
    ) -> (f64, Option<f64>) {
        let (score, valid) = state.eval();
        // let (score, valid) = state.naive_eval(self);
        // let (score, valid) = state.parry_eval(self);
        (-score, if valid { Some(-score) } else { None })
    }

    fn apply_and_eval(
        &self,
        state: &mut Self::State,
        mov: &Self::Move,
        progress_ratio: f64,
        best_score: f64,
        valid_best_score: f64,
        _prev_score: f64,
    ) -> (f64, Option<f64>) {
        self.apply(state, mov);
        state.eval_mut(self);
        self.eval(state, progress_ratio, best_score, valid_best_score)
    }

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl Rng,
        _progress_ratio: f64,
    ) -> Self::Move {
        let s = &self.0;
        let stage_valid = self.stage_valid();

        // let scale_x = (self.stage_valid.width() / 5.0).max(15.0);
        // let scale_y = (self.stage_valid.height() / 5.0).max(15.0);
        let scale_x = 15.0;
        let scale_y = 15.0;

        match rng.gen_range(0..=0) {
            0 => loop {
                let id = rng.gen_range(0..state.placement.len());
                // if state.impact[id] == 0.0 {
                //     continue;
                // }

                // let theta = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
                // let len = rng.gen_range(0.1..=scale);
                // let d = Vector2D::from_angle_and_length(Angle::radians(theta), len);
                let d = vec2(
                    rng.gen_range(-scale_x..=scale_x),
                    rng.gen_range(-scale_y..=scale_y),
                );

                let new_pos = state.placement[id] + d;
                let new_pos = point2(
                    new_pos.x.clamp(stage_valid.min.x, stage_valid.max.x),
                    new_pos.y.clamp(stage_valid.min.y, stage_valid.max.y),
                );

                if state.placement[id] == new_pos {
                    continue;
                }

                if state
                    .placement
                    .iter()
                    .enumerate()
                    .any(|(i, p)| i != id && p.distance_to(new_pos) < 10.0)
                {
                    continue;
                }

                break Move::Change {
                    id,
                    new_pos,
                    old_pos: state.placement[id],
                };
            },

            1 => loop {
                let i = rng.gen_range(0..state.placement.len());
                let j = rng.gen_range(0..state.placement.len());
                if i != j {
                    break Move::Swap { i, j };
                }
            },
            _ => unreachable!(),
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        match mov {
            Move::Change { id, new_pos, .. } => {
                state.change(self, *id, *new_pos);
            }
            Move::Swap { i, j } => {
                let pi = state.placement[*i];
                let pj = state.placement[*j];
                state.change(self, *i, pj);
                state.change(self, *j, pi);
            }
        }
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        match mov {
            Move::Change { id, old_pos, .. } => {
                state.change(self, *id, *old_pos);
            }
            Move::Swap { i, j } => {
                let pi = state.placement[*i];
                let pj = state.placement[*j];
                state.change(self, *i, pj);
                state.change(self, *j, pi);
            }
        }
    }
}

struct Solver2(common::Problem, Option<common::Solution>);

#[derive(Clone)]
struct State2 {
    board: Board,
}

impl saru::Annealer for Solver2 {
    type State = State2;

    type Move = Move;

    fn init_state(&self, rng: &mut impl Rng) -> Self::State {
        let mut board = Board::new(0, self.0.clone());

        if let Some(initial_solution) = &self.1 {
            for (i, p) in initial_solution.placements.iter().enumerate() {
                board.try_place(i, p.position).unwrap();
            }
        } else {
            for i in 0..board.prob.musicians.len() {
                loop {
                    let x: f64 = rng.gen_range(board.prob.stage.min.x..=board.prob.stage.max.x);
                    let y: f64 = rng.gen_range(board.prob.stage.min.y..=board.prob.stage.max.y);
                    if board.try_place(i, Point::new(x, y)).is_ok() {
                        break;
                    }
                }
            }
        }

        State2 { board }
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        (init_score.abs() * 0.1).max(1e8)
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
        _progress_ratio: f64,
    ) -> Self::Move {
        // let scale_x = (self.stage_valid.width() / 5.0).max(15.0);
        // let scale_y = (self.stage_valid.height() / 5.0).max(15.0);
        let scale_x = 15.0;
        let scale_y = 15.0;

        match rng.gen_range(0..=0) {
            0 => loop {
                let id = rng.gen_range(0..state.board.musicians().len());
                let d = vec2(
                    rng.gen_range(-scale_x..=scale_x),
                    rng.gen_range(-scale_y..=scale_y),
                );

                let old_pos = state.board.musicians()[id].unwrap().to_point();
                let new_pos = old_pos + d;
                let new_pos = point2(
                    new_pos
                        .x
                        .clamp(state.board.prob.stage.min.x, state.board.prob.stage.max.x),
                    new_pos
                        .y
                        .clamp(state.board.prob.stage.min.y, state.board.prob.stage.max.y),
                );

                if new_pos == old_pos {
                    continue;
                }

                if !state.board.can_place(id, new_pos) {
                    continue;
                }

                break Move::Change {
                    id,
                    new_pos,
                    old_pos,
                };
            },

            // 1 => loop {
            //     let i = rng.gen_range(0..state.placement.len());
            //     let j = rng.gen_range(0..state.placement.len());
            //     if i != j {
            //         break Move::Swap { i, j };
            //     }
            // },
            _ => unreachable!(),
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        match mov {
            Move::Change { id, new_pos, .. } => {
                state.board.unplace(*id);
                state.board.try_place(*id, *new_pos).unwrap();
            }
            Move::Swap { .. } => todo!(),
        }
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        match mov {
            Move::Change { id, old_pos, .. } => {
                state.board.unplace(*id);
                state.board.try_place(*id, *old_pos).unwrap();
            }
            Move::Swap { .. } => todo!(),
        }
    }
}

// struct PreSolver(Solver);

// #[derive(Clone)]
// struct PreState {
//     placement: Vec<Point2D<f64>>,
//     // atendee_to_musician: Vec<(Vec<f64>, f64)>,
// }

// impl saru::Annealer for PreSolver {
//     type State = PreState;

//     type Move = Move;

//     fn init_state(&self, rng: &mut impl Rng) -> Self::State {
//         let s = &self.0;
//         let n = s.musicians.len();

//         let mut placement = vec![];

//         for _ in 0..n {
//             loop {
//                 let x = rng.gen_range(s.stage_valid.min_x()..=s.stage_valid.max_x());
//                 let y = rng.gen_range(s.stage_valid.min_y()..=s.stage_valid.max_y());

//                 let p = point2(x, y);
//                 if placement.iter().any(|q| p.distance_to(*q) < 10.0) {
//                     continue;
//                 }
//                 placement.push(point2(x, y));
//                 break;
//             }
//         }

//         PreState { placement }
//     }

//     fn start_temp(&self, init_score: f64) -> f64 {
//         (init_score.abs() / 10.0).max(1e8)
//     }

//     fn eval(
//         &self,
//         state: &Self::State,
//         _progress_ratio: f64,
//         _best_score: f64,
//         _valid_best_score: f64,
//     ) -> (f64, Option<f64>) {
//         todo!()
//     }

//     fn neighbour(
//         &self,
//         state: &mut Self::State,
//         rng: &mut impl Rng,
//         progress_ratio: f64,
//     ) -> Self::Move {
//         todo!()
//     }

//     fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
//         todo!()
//     }

//     fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
//         todo!()
//     }
// }

#[argopt::cmd]
fn main(
    /// time limit in seconds
    #[opt(long, default_value = "10.0")]
    time_limit: f64,
    /// number of threads
    #[opt(long, default_value = "1")]
    threads: usize,
    #[opt(long)] initial_solution: Option<PathBuf>,
    /// problem id
    problem_id: u32,
) -> Result<()> {
    let client = Client::new();

    // let problem = get_problem_from_file(problem_id)?;
    let problem = client.get_problem(problem_id)?;

    eprintln!("Musicians: {}", problem.musicians.len());
    eprintln!("Atendees:  {}", problem.attendees.len());

    let initial_solution: Option<Solution> = if let Some(path) = initial_solution {
        let s = std::fs::read_to_string(path)?;
        let raw_solution: RawSolution = serde_json::from_str(&s)?;
        Some(raw_solution.into())
    } else {
        None
    };

    let solver = Solver2(problem, initial_solution);

    let solution = saru::annealing(
        &solver,
        &saru::AnnealingOptions {
            time_limit,
            limit_temp: 0.1,
            restart: 0,
            threads,
            silent: false,
            header: "".to_string(),
        },
        rand::thread_rng().gen(),
    );

    eprintln!("Statistics:");
    eprintln!("Score:         {}", -solution.score);
    eprintln!("Musicians:     {}", solver.0.musicians.len());
    eprintln!("Atendees:      {}", solver.0.attendees.len());
    eprintln!("Stage area:    {}", solver.0.stage.area());

    // let lx = (solver.0.stage_valid().width() / 10.0).floor();
    // let ly = (solver.stage_valid().height() / 10.0).floor();
    // eprintln!("Stage lattice: {}", (lx * ly) as i64);
    // eprintln!(
    //     "Lattice/mucs:  {}",
    //     lx * ly / solver.0.musicians.len() as f64
    // );

    let Some(state) = solution.state else {
        anyhow::bail!("Valid solution not found")
    };

    let score = -solution.score;

    let solution: Solution = state.board.try_into()?;
    let raw_solution = RawSolution::from(solution.clone());

    {
        if !std::path::Path::new("results").is_dir() {
            std::fs::create_dir_all("results")?;
        }
        let file_name = format!("results/sol-{problem_id:03}-{}.json", score);
        std::fs::write(file_name, format!("{}", serde_json::json!(raw_solution)))?;
    }

    if score <= 0.0 {
        anyhow::bail!("Positive score not found");
    }

    let resp = client.post_submission(problem_id, solution)?;
    eprintln!("Submitted: {:?}", resp);

    Ok(())
}
