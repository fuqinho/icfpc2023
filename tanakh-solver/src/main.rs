use anyhow::Result;
use euclid::{default::*, point2, rect, size2, vec2};
use lyon_geom::LineSegment;
use rand::Rng;

use tanakh_solver::api::{get_problem, submit, Placement, Problem};

const PROBLEM_PATH: &str = "../problems";
const EPS: f64 = 1e-6;

#[allow(unused)]
fn get_problem_from_file(problem_id: u32) -> Result<Problem> {
    let s = std::fs::read_to_string(&format!("{PROBLEM_PATH}/{problem_id}.json"))?;
    Ok(serde_json::from_str(&s)?)
}

struct Solver {
    #[allow(unused)]
    room_size: Size2D<f64>,
    #[allow(unused)]
    stage: Rect<f64>,
    stage_valid: Rect<f64>,
    musicians: Vec<usize>,
    attendees: Vec<Atendee>,
}

struct Atendee {
    pos: Point2D<f64>,
    tastes: Vec<f64>,
}

#[derive(Clone)]
struct State {
    placement: Vec<Point2D<f64>>,
    attendee_to_musician: Vec<Vec<(LineSegment<f64>, f64)>>,
    musician_to_musician: Vec<Vec<f64>>,
    distance_penalty: f64,
}

impl Solver {
    fn from_problem(p: &Problem) -> Self {
        Solver {
            room_size: size2(p.room_width, p.room_height),
            stage: rect(
                p.stage_bottom_left[0],
                p.stage_bottom_left[1],
                p.stage_width,
                p.stage_height,
            ),
            stage_valid: rect(
                p.stage_bottom_left[0] + 10.0,
                p.stage_bottom_left[1] + 10.0,
                p.stage_width - 20.0,
                p.stage_height - 20.0,
            ),
            musicians: p.musicians.clone(),
            attendees: p
                .attendees
                .iter()
                .map(|a| Atendee {
                    pos: point2(a.x, a.y),
                    tastes: a.tastes.clone(),
                })
                .collect(),
        }
    }
}

impl State {
    fn new(placement: Vec<Point2D<f64>>, s: &Solver) -> Self {
        let mut ret = Self {
            placement,
            attendee_to_musician: vec![
                vec![
                    (
                        LineSegment {
                            from: point2(0.0, 0.0),
                            to: point2(0.0, 0.0),
                        },
                        0.0
                    );
                    s.musicians.len()
                ];
                s.attendees.len()
            ],
            musician_to_musician: vec![vec![0.0; s.musicians.len()]; s.musicians.len()],
            distance_penalty: 0.0,
        };

        for i in 0..s.attendees.len() {
            for k in 0..ret.placement.len() {
                let line = LineSegment {
                    from: s.attendees[i].pos,
                    to: ret.placement[k],
                };

                let mut block_dist = f64::MAX;
                for l in 0..ret.placement.len() {
                    if l == k {
                        continue;
                    }
                    block_dist = block_dist.min(line.distance_to_point(ret.placement[l]));
                }

                ret.attendee_to_musician[i][k] = (line, block_dist);
            }
        }

        ret
    }

    fn change(&mut self, s: &Solver, k: usize, new_pos: Point2D<f64>) {
        for i in 0..s.attendees.len() {
            let line = LineSegment {
                from: s.attendees[i].pos,
                to: new_pos,
            };

            let mut block_dist = f64::MAX;
            for l in 0..self.placement.len() {
                if l == k {
                    continue;
                }
                block_dist = block_dist.min(line.distance_to_point(self.placement[l]));
            }

            self.attendee_to_musician[i][k] = (line, block_dist);
        }

        let old_pos = self.placement[k];
        self.placement[k] = new_pos;

        for i in 0..s.attendees.len() {
            for l in 0..self.placement.len() {
                if l == k {
                    continue;
                }

                let line = self.attendee_to_musician[i][l].0;

                let d = line.distance_to_point(old_pos);
                if self.attendee_to_musician[i][l].1 <= d + EPS {
                    continue;
                }

                // changed minimum dist. recalculate all.
                let mut block_dist = f64::MAX;
                for m in 0..self.placement.len() {
                    if l == m {
                        continue;
                    }
                    block_dist = block_dist.min(line.distance_to_point(self.placement[m]));
                }

                self.attendee_to_musician[i][l].1 = block_dist;
            }
        }
    }

    fn eval(&self, s: &Solver) -> (f64, bool) {
        let mut score = 0.0;

        for i in 0..s.attendees.len() {
            for k in 0..self.placement.len() {
                if self.attendee_to_musician[i][k].1 > 5.0 {
                    let d = self.attendee_to_musician[i][k].0.length();
                    let taste = s.attendees[i].tastes[s.musicians[k]];
                    score += (1_000_000.0 * taste / d.powi(2)).ceil();
                }
            }
        }

        let penalty = self.distance_penalty;

        (score - penalty, penalty == 0.0)
    }

    fn naive_eval(&self, s: &Solver) -> (f64, bool) {
        let mut ret = 0.0;

        for i in 0..s.attendees.len() {
            for k in 0..self.placement.len() {
                let line = LineSegment {
                    from: s.attendees[i].pos,
                    to: self.placement[k],
                };

                let mut block_dist = f64::MAX;
                for l in 0..self.placement.len() {
                    if l == k {
                        continue;
                    }
                    block_dist = block_dist.min(line.distance_to_point(self.placement[l]));
                }

                if block_dist > 5.0 {
                    let taste = s.attendees[i].tastes[s.musicians[k]];
                    // ret += (1_000_000.0 * taste / line.length().powi(2)).ceil();
                    if taste > 0.0 {
                        ret += (1_000_000.0 * taste / line.length().powi(2)).ceil();
                    } else {
                        ret += (1_000_000.0 * (block_dist / 5.0).min(4.0) * taste
                            / line.length().powi(2))
                        .ceil();
                    }
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

impl saru::Annealer for Solver {
    type State = State;

    type Move = Move;

    fn init_state(&self, rng: &mut impl Rng) -> Self::State {
        let n = self.musicians.len();

        let mut placement = vec![];

        for _ in 0..n {
            loop {
                let x = rng.gen_range(self.stage_valid.x_range());
                let y = rng.gen_range(self.stage_valid.y_range());

                let p = point2(x, y);
                if placement.iter().any(|q| p.distance_to(*q) < 10.0) {
                    continue;
                }
                placement.push(point2(x, y));
                break;
            }
        }

        State::new(placement, self)
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        (init_score.abs() / 10.0).max(1e6)
    }

    fn eval(
        &self,
        state: &Self::State,
        _progress_ratio: f64,
        _best_score: f64,
        _valid_best_score: f64,
    ) -> (f64, Option<f64>) {
        let (score, valid) = state.eval(self);
        // let (score, valid) = state.naive_eval(self);
        (-score, if valid { Some(-score) } else { None })
    }

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl Rng,
        _progress_ratio: f64,
    ) -> Self::Move {
        match rng.gen_range(0..=0) {
            0 => loop {
                let id = rng.gen_range(0..state.placement.len());
                let dx = rng.gen_range(-50.0..=50.0);
                let dy = rng.gen_range(-50.0..=50.0);
                let new_pos = state.placement[id] + vec2(dx, dy);
                let new_pos = point2(
                    new_pos
                        .x
                        .clamp(self.stage_valid.min_x(), self.stage_valid.max_x()),
                    new_pos
                        .y
                        .clamp(self.stage_valid.min_y(), self.stage_valid.max_y()),
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
                if i == j {
                    continue;
                }

                break Move::Swap { i, j };
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

#[argopt::cmd]
fn main(
    /// time limit in seconds
    #[opt(long, default_value = "10.0")]
    time_limit: f64,
    /// number of threads
    #[opt(long, default_value = "1")]
    threads: usize,
    /// problem id
    problem_id: u32,
) -> Result<()> {
    // let problem = get_problem_from_file(problem_id)?;
    let problem = get_problem(problem_id)?;

    eprintln!("Musicians: {}", problem.musicians.len());
    eprintln!("Atendees:  {}", problem.attendees.len());

    let solver = Solver::from_problem(&problem);

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
    eprintln!("Musicians:     {}", problem.musicians.len());
    eprintln!("Atendees:      {}", problem.attendees.len());
    eprintln!("Stage area:    {}", solver.stage.area());

    let lx = ((solver.stage_valid.max_x() - solver.stage_valid.min_x()) / 10.0).floor();
    let ly = ((solver.stage_valid.max_y() - solver.stage_valid.min_y()) / 10.0).floor();
    eprintln!("Stage lattice: {}", (lx * ly) as i64);
    eprintln!("Lattice/mucs:  {}", lx * ly / solver.musicians.len() as f64);

    let Some(state) = solution.state else {
        anyhow::bail!("Valid solution not found")
    };

    if solution.score >= 0.0 {
        anyhow::bail!("Positive score not found");
    }

    let placements = state
        .placement
        .iter()
        .map(|p| Placement { x: p.x, y: p.y })
        .collect::<Vec<_>>();

    {
        if !std::path::Path::new("results").is_dir() {
            std::fs::create_dir_all("results")?;
        }
        let file_name = format!("results/sol-{problem_id:03}-{}.json", -solution.score);
        std::fs::write(
            file_name,
            format!(
                "{}",
                serde_json::json!({ "problem_id": problem_id, "placements": placements })
            ),
        )?;
    }

    let resp = submit(problem_id, &placements)?;
    eprintln!("Submitted: {}", resp.0);

    Ok(())
}
