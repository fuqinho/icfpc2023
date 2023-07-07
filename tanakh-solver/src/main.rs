use anyhow::Result;
use euclid::{default::*, point2, rect, size2, vec2};
use lyon_geom::LineSegment;
use rand::Rng;

use tanakh_solver::api::{get_problem, submit, Placement, Problem};

const PROBLEM_PATH: &str = "../problems";

fn get_problem_from_file(problem_id: u32) -> Result<Problem> {
    let s = std::fs::read_to_string(&format!("{PROBLEM_PATH}/{problem_id}.json"))?;
    Ok(serde_json::from_str(&s)?)
}

struct Solver {
    room_size: Size2D<f64>,
    stage: Rect<f64>,
    stage_valid: Rect<f64>,
    musicians: Vec<usize>,
    attendees: Vec<Atendee>,
}

struct Atendee {
    pos: Point2D<f64>,
    tastes: Vec<f64>,
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

struct Move {
    id: usize,
    new_pos: Point2D<f64>,
    old_pos: Point2D<f64>,
}

impl saru::Annealer for Solver {
    type State = Vec<Point2D<f64>>;

    type Move = Move;

    fn init_state(&self, rng: &mut impl Rng) -> Self::State {
        let n = self.musicians.len();

        let mut ret = vec![];

        for _ in 0..n {
            let x = rng.gen_range(self.stage_valid.min_x()..=self.stage_valid.max_x());
            let y = rng.gen_range(self.stage_valid.min_y()..=self.stage_valid.max_y());
            ret.push(point2(x, y));
        }

        ret
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        // init_score.abs() / 10.0
        1e9
    }

    fn eval(
        &self,
        state: &Self::State,
        _progress_ratio: f64,
        _best_score: f64,
        _valid_best_score: f64,
    ) -> (f64, Option<f64>) {
        let mut score = 0.0;

        for i in 0..self.attendees.len() {
            for k in 0..state.len() {
                let line = LineSegment {
                    from: self.attendees[i].pos,
                    to: state[k],
                };

                let mut block_dist = f64::MAX;
                for l in 0..state.len() {
                    if l == k {
                        continue;
                    }
                    block_dist = block_dist.min(line.distance_to_point(state[l]));
                }

                if block_dist > 5.0 {
                    let d = line.length();
                    let taste = self.attendees[i].tastes[self.musicians[k]];
                    score += (1_000_000.0 * taste / d.powi(2)).ceil();
                }
            }
        }

        let mut penalty = 0.0;

        for i in 0..state.len() {
            for j in i + 1..state.len() {
                let d = (state[i] - state[j]).length();
                if d < 10.0 {
                    penalty += 1_000_000_000.0 / (d + 1.0);
                }
            }
        }

        let score = score - penalty;

        (-score, if penalty == 0.0 { Some(-score) } else { None })
    }

    fn neighbour(
        &self,
        state: &mut Self::State,
        rng: &mut impl Rng,
        _progress_ratio: f64,
    ) -> Self::Move {
        let id = rng.gen_range(0..state.len());
        let dx = rng.gen_range(-10.0..10.0);
        let dy = rng.gen_range(-10.0..10.0);
        let new_pos =
            (state[id] + vec2(dx, dy)).clamp(self.stage_valid.origin, self.stage_valid.max());
        // let new_pos = point2(
        //     new_pos
        //         .x
        //         .clamp(self.stage_valid.min_x(), self.stage_valid.max_x()),
        //     new_pos
        //         .y
        //         .clamp(self.stage_valid.min_y(), self.stage_valid.max_y()),
        // );

        let old_pos = state[id];
        Move {
            id,
            new_pos,
            old_pos,
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        state[mov.id] = mov.new_pos;
    }

    fn unapply(&self, state: &mut Self::State, mov: &Self::Move) {
        state[mov.id] = mov.old_pos;
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
        .iter()
        .map(|p| Placement { x: p.x, y: p.y })
        .collect::<Vec<_>>();

    println!("{}", serde_json::json!({ "placements": placements }));

    let resp = submit(problem_id, &placements)?;
    eprintln!("Submitted: {}", resp.0);

    Ok(())
}
