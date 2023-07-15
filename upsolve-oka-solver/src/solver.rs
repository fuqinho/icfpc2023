use std::f64::consts::PI;

use common::{board_options::BoardOptions, float, Problem};
use log::info;
use lyon_geom::{Box2D, LineSegment, Vector};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::pretty::pretty;
use anyhow::{bail, Result};

type Board = common::board::Board<float::F64>;
type P = Vector<f64>;

const MAX_TEMP: f64 = 10_000_000.0;
const MIN_TEMP: f64 = 0.0;
const TEMP_FUNC_POWER: f64 = 2.0;

const MAX_MOVE_DIST: f64 = 40.0;
const MIN_MOVE_DIST: f64 = 5.0;

pub struct Solver {
    board: Board,
    num_iter: usize,
    rng: SmallRng,

    forbidden_area: Box2D<f64>,

    visible_musicians_count: usize,

    is_visible: Vec<bool>,
    musicians: Vec<P>,
}

impl Solver {
    pub fn new(problem_id: u32, problem: Problem, num_iter: usize) -> Self {
        if problem.stage.min.x > 0. || problem.stage.min.y > 0. {
            panic!("Unsupported stage min: {:?}", problem.stage.min);
        }

        // Parameters
        let placed_musicians_ratio = 0.5;
        let important_attendees_ratio = 0.2;
        let important_musician_range = 300.0;

        let options = BoardOptions::default()
            .with_important_attendees_ratio(important_attendees_ratio)
            .with_important_musician_range(important_musician_range);

        let board =
            Board::new_with_options(problem_id, problem, "upsolve-oka-solver", false, options);

        let rng = SmallRng::seed_from_u64(0);

        let d =
            (board.prob.stage.min - board.prob.stage.max).normalize() * important_musician_range;
        let forbidden_area = Box2D::new(board.prob.stage.min, board.prob.stage.max + d);

        let visible_musicians_count =
            (board.musicians().len() as f64 * placed_musicians_ratio) as usize;
        let is_visible = vec![false; board.musicians().len()];
        let musicians = vec![board.prob.stage.min.to_vector(); board.musicians().len()];

        Self {
            board,
            num_iter,
            rng,
            forbidden_area,
            visible_musicians_count,
            is_visible,
            musicians,
        }
    }

    pub fn solve(&mut self) -> Board {
        for i in 0..self.board.musicians().len() {
            self.board.set_volume(i, 10.0);
        }
        // Initialize with random positions
        for i in 0..self.visible_musicians_count {
            loop {
                let p = self.random_place();

                self.move_musician_to(i, p).unwrap();

                if self.set_visibility(i, true).is_ok() {
                    break;
                }
            }
        }

        for iter in 0..=self.num_iter {
            self.step(iter);

            if iter % (self.num_iter / 100) == 0 {
                let temp = self.temp(iter);
                info!(
                    "{:>3}% iter: {:>10}  score: {:>14}  temp: {:>10}",
                    (iter * 100) / self.num_iter,
                    pretty(iter as i64),
                    pretty(self.board.score() as i64),
                    pretty(temp as i64),
                );
            }
        }

        // Place remaining musicians randomly
        let mut remaining_musicians = vec![];
        for i in 0..self.board.musicians().len() {
            if self.is_visible[i] {
                continue;
            }
            remaining_musicians.push(i);
        }

        for x in ((self.board.prob.stage.min.x as usize)..(self.board.prob.stage.max.x as usize))
            .step_by(10)
        {
            if remaining_musicians.is_empty() {
                break;
            }

            for y in ((self.board.prob.stage.min.y as usize)
                ..(self.board.prob.stage.max.y as usize))
                .step_by(10)
            {
                if remaining_musicians.is_empty() {
                    break;
                }

                let p = P::new(x as f64, y as f64);

                if self.forbidden_area.contains(p.to_point()) {
                    continue;
                }

                let m = remaining_musicians.last().unwrap();
                self.move_musician_to(*m, p).unwrap();

                let prev_score = self.board.score();

                if self.set_visibility(*m, true).is_err() {
                    continue;
                }
                if self.board.score() < prev_score {
                    self.set_visibility(*m, false).unwrap();
                    continue;
                }

                remaining_musicians.pop();
            }
        }

        self.board.hungarian();

        self.board.clone()
    }

    fn set_visibility(&mut self, m: usize, visible: bool) -> Result<()> {
        if self.is_visible[m] == visible {
            return Ok(());
        }

        if visible {
            self.board.try_place(m, self.musicians[m].to_point())?;
            self.is_visible[m] = true;
        } else {
            self.board.unplace(m);
            self.is_visible[m] = false;
        }

        Ok(())
    }

    fn move_musician_to(&mut self, m: usize, p: P) -> Result<()> {
        if !self.board.prob.stage.contains(p.to_point()) {
            bail!("Out of stage");
        }
        if !self.is_visible[m] {
            self.musicians[m] = p;
            return Ok(());
        }

        let orig = self.board.musicians()[m].unwrap().0;

        assert_eq!(orig, self.musicians[m]);

        self.board.unplace(m);

        if let Err(e) = self.board.try_place(m, p.to_point()) {
            self.board.try_place(m, orig.to_point()).unwrap();
            return Err(e);
        }

        self.musicians[m] = p;
        Ok(())
    }

    fn temp(&self, iter: usize) -> f64 {
        let max_temp = MAX_TEMP;
        let min_temp = MIN_TEMP;

        let r = iter as f64 / self.num_iter as f64;

        min_temp + (max_temp - min_temp) * (1. - r).powf(TEMP_FUNC_POWER)
    }

    fn step(&mut self, iter: usize) {
        let score = self.board.score();

        let action = self.random_action(iter);

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

        let accept_prob = 1.0 + improve / temp;

        self.rng.gen_range(0.0..1.0) < accept_prob
    }

    fn apply(&mut self, action: Action) -> bool {
        match action {
            Action::Swap(x, y) => {
                let xp = self.musicians[x];
                let yp = self.musicians[y];

                let x_vis = self.is_visible[x];
                let y_vis = self.is_visible[y];

                self.set_visibility(x, false).unwrap();
                self.set_visibility(y, false).unwrap();

                self.move_musician_to(x, yp).unwrap();
                self.move_musician_to(y, xp).unwrap();

                self.set_visibility(x, y_vis).unwrap();
                self.set_visibility(y, x_vis).unwrap();

                true
            }
            Action::MoveRandom(m, _, p) => self.move_musician_to(m, p).is_ok(),
            Action::MoveDir(m, dir) => {
                let dest = self.move_to_dir(self.musicians[m], dir);
                self.move_musician_to(m, dest).is_ok()
            }
            Action::Hungarian => {
                self.board.hungarian();

                for (m, p) in self.board.musicians().iter().enumerate() {
                    if let Some((p, _)) = p {
                        self.musicians[m] = *p;
                        self.is_visible[m] = true;
                    } else {
                        self.is_visible[m] = false;
                    }
                }

                true
            }
        }
    }

    fn random_action(&mut self, iter: usize) -> Action {
        if self.rng.gen_range(0..1_000_000) == 0 {
            return Action::Hungarian;
        }

        loop {
            let v = self.rng.gen_range(0..100);

            match v {
                // Swap random two musicianss
                0..=9 => loop {
                    let x = self.random_musician();
                    let y = self.random_musician();

                    if x == y {
                        continue;
                    }
                    if !self.is_visible[x] && !self.is_visible[y] {
                        continue;
                    }

                    return Action::Swap(x, y);
                },
                // Move a musician to a random place
                10..=19 => {
                    let x = self.random_visible_musician();
                    let orig = self.musicians[x];
                    let p = self.random_place();

                    return Action::MoveRandom(x, orig, p);
                }
                // Move a musician to a random direction
                20..=99 => {
                    let x = self.random_visible_musician();
                    let dir = self.random_direction(iter);

                    return Action::MoveDir(x, dir);
                }
                _ => continue,
            }
        }
    }

    fn random_direction(&mut self, iter: usize) -> P {
        let d: f64 = self.rng.gen_range(0.0..1.0);
        let r = self.rng.gen_range(-1.0..1.0) * PI;

        let (x, y) = r.sin_cos();

        let dd = d.powi(2);

        let max_dist = MAX_MOVE_DIST
            - (MAX_MOVE_DIST - MIN_MOVE_DIST) * (MAX_TEMP - self.temp(iter))
                / (MAX_TEMP - MIN_TEMP);

        P::new(x * max_dist * dd, y * max_dist * dd)
    }

    fn random_musician(&mut self) -> usize {
        self.rng.gen_range(0..self.board.musicians().len())
    }

    fn random_visible_musician(&mut self) -> usize {
        loop {
            let x = self.rng.gen_range(0..self.board.musicians().len());
            if self.is_visible[x] {
                return x;
            }
        }
    }

    fn random_place(&mut self) -> P {
        loop {
            let x = self
                .rng
                .gen_range(self.board.prob.stage.min.x..self.board.prob.stage.max.x);
            let y = self
                .rng
                .gen_range(self.board.prob.stage.min.y..self.board.prob.stage.max.y);

            let p = P::new(x, y);

            if !self.forbidden_area.contains(p.to_point()) {
                return p;
            }
        }
    }

    fn move_to_dir(&self, p: P, dir: P) -> P {
        let dest = p + dir;

        if !self.forbidden_area.contains(dest.to_point()) {
            return dest;
        }

        let l = LineSegment {
            from: p.to_point(),
            to: dest.to_point(),
        };

        if let Some(q) = l.horizontal_line_intersection(self.forbidden_area.max.y) {
            let dir = dest - q.to_vector();

            let nq = self.translate_point_on_forbideen_bounding_box(q.to_vector());
            let ndir = P::new(-dir.y, dir.x);

            return nq + ndir;
        } else if let Some(q) = l.vertical_line_intersection(self.forbidden_area.max.x) {
            let dir = dest - q.to_vector();

            let nq = self.translate_point_on_forbideen_bounding_box(q.to_vector());
            let ndir = P::new(dir.y, -dir.x);

            return nq + ndir;
        } else {
            panic!("No intersection found: {:?}", l);
        }
    }

    fn translate_point_on_forbideen_bounding_box(&self, p: P) -> P {
        let (fx, fy) = (self.forbidden_area.max.x, self.forbidden_area.max.y);

        if (fy - p.y).abs() < 1e-6 {
            P::new(fx, p.x / fx * fy)
        } else if (fx - p.x).abs() < 1e-6 {
            P::new(p.y / fy * fx, fy)
        } else {
            panic!("Not on bounding box: {:?}", p);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Swap(usize, usize),
    MoveRandom(usize, /* from */ P, /* to */ P),
    MoveDir(usize, P),
    Hungarian,
}

impl Action {
    fn invert(&self) -> Action {
        match self {
            Action::Swap(x, y) => Action::Swap(*x, *y),
            Action::MoveRandom(m, orig, p) => Action::MoveRandom(*m, *p, *orig),
            Action::MoveDir(m, p) => Action::MoveDir(*m, -*p),
            Action::Hungarian => Action::Hungarian,
        }
    }
}
