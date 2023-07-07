use anyhow::Result;
use euclid::{default::*, point2, rect, size2, vec2};
use lyon_geom::LineSegment;
use rand::Rng;

use tanakh_solver::api::{get_problem, submit, Placement, Problem};

const PROBLEM_PATH: &str = "../problems";
const EPS: f64 = 1e-9;

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
}

struct Move {
    id: usize,
    new_pos: Point2D<f64>,
    old_pos: Point2D<f64>,
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

    fn change(&mut self, s: &Solver, k: usize) {
        for i in 0..s.attendees.len() {
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

            self.attendee_to_musician[i][k] = (line, block_dist);
        }

        for i in 0..s.attendees.len() {
            for l in 0..self.placement.len() {
                if l == k {
                    continue;
                }

                let line = self.attendee_to_musician[i][l].0;

                let d = line.distance_to_point(self.placement[k]);
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

        let mut penalty = 0.0;

        for i in 0..self.placement.len() {
            for j in i + 1..self.placement.len() {
                let d = (self.placement[i] - self.placement[j]).length();
                if d < 10.0 {
                    penalty += 1_000_000_000.0 / (d + 1.0);
                }
            }
        }

        (score - penalty, penalty == 0.0)
    }
}

impl saru::Annealer for Solver {
    type State = State;

    type Move = Move;

    fn init_state(&self, rng: &mut impl Rng) -> Self::State {
        let n = self.musicians.len();

        let mut placement = vec![];

        for _ in 0..n {
            let x = rng.gen_range(self.stage_valid.min_x()..=self.stage_valid.max_x());
            let y = rng.gen_range(self.stage_valid.min_y()..=self.stage_valid.max_y());
            placement.push(point2(x, y));
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
        (-score, if valid { Some(-score) } else { None })
    }

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl Rng,
        _progress_ratio: f64,
    ) -> Self::Move {
        let id = rng.gen_range(0..state.placement.len());
        let dx = rng.gen_range(-10.0..10.0);
        let dy = rng.gen_range(-10.0..10.0);
        let new_pos = state.placement[id] + vec2(dx, dy);
        let new_pos = point2(
            new_pos
                .x
                .clamp(self.stage_valid.min_x(), self.stage_valid.max_x()),
            new_pos
                .y
                .clamp(self.stage_valid.min_y(), self.stage_valid.max_y()),
        );

        // eprintln!("* {new_pos:?}");

        Move {
            id,
            new_pos,
            old_pos: state.placement[id],
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        state.placement[mov.id] = mov.new_pos;
        state.change(self, mov.id);
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        state.placement[mov.id] = mov.old_pos;
        state.change(self, mov.id);
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

    eprintln!("Score: {}", solution.score);

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

    println!("{}", serde_json::json!({ "placements": placements }));

    let resp = submit(problem_id, &placements)?;
    eprintln!("Submitted: {}", resp.0);

    Ok(())
}
