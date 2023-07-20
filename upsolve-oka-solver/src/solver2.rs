use std::f64::consts::PI;

use common::{board_options::BoardOptions, float, Problem, Solution};
use log::info;
use lyon_geom::{Box2D, LineSegment, Vector};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{params::Params, pretty::pretty};
use anyhow::{bail, Result};

type Board = common::board::Board<float::F64>;
type P = Vector<f64>;

pub struct Solver2 {
    orig_problem: Problem,

    board: Board,
    num_iter: usize,
    rng: SmallRng,

    forbidden_area: Box2D<f64>,

    visible_musicians_count: usize,

    is_visible: Vec<bool>,
    musicians: Vec<P>,

    params: Params,
    initial_solution: Option<Solution>,
}

impl Solver2 {
    pub fn new(
        problem_id: u32,
        problem: Problem,
        num_iter: usize,
        params: Params,
        initial_solution: Option<Solution>,
    ) -> Self {
        if problem.stage.min.x > 0. || problem.stage.min.y > 0. {
            panic!("Unsupported stage min: {:?}", problem.stage.min);
        }

        let orig_problem = problem.clone();

        let options = BoardOptions::default()
            .with_important_attendees_ratio(params.important_attendees_ratio)
            .with_important_musician_range(params.important_musician_range);

        let board =
            Board::new_with_options(problem_id, problem, "upsolve-oka-solver", false, options);

        let rng = SmallRng::seed_from_u64(0);

        let d = (board.prob.stage.max - board.prob.stage.min).normalize()
            * params.important_musician_range
            * params.forbidden_area_coeff;
        let forbidden_area = Box2D::new(board.prob.stage.min, board.prob.stage.max - d);

        let is_visible = vec![false; board.musicians().len()];
        let musicians = vec![board.prob.stage.min.to_vector(); board.musicians().len()];

        Self {
            orig_problem,
            board,
            num_iter,
            rng,
            forbidden_area,
            visible_musicians_count: 0,
            is_visible,
            musicians,
            params,
            initial_solution,
        }
    }

    pub fn initialize(&mut self) {
        for i in 0..self.board.musicians().len() {
            self.board.set_volume(i, 10.0);
        }

        let initial_visible_musicians_count =
            (self.board.musicians().len() as f64 * self.params.placed_musicians_ratio) as usize;

        if let Some(initial_solution) = self.initial_solution.clone() {
            for (i, p) in initial_solution.placements.iter().enumerate() {
                self.move_musician_to(i, p.position.to_vector()).unwrap();

                self.set_visibility(i, true).unwrap();
            }
            let mut scores = vec![];
            for i in 0..self.board.musicians().len() {
                scores.push((self.board.contribution2(i) as i64, i));
            }
            scores.sort();

            for i in 0..self.board.musicians().len() - initial_visible_musicians_count {
                self.set_visibility(scores[i].1, false).unwrap();
            }

            info!("Initial solution score: {}", self.board.score());

            return;
        }
        // Initialize with random positions
        for i in 0..initial_visible_musicians_count {
            loop {
                let p = self.random_place();

                self.move_musician_to(i, p).unwrap();

                if self.set_visibility(i, true).is_ok() {
                    break;
                }
            }
        }
    }

    pub fn solve(&mut self) -> Board {
        self.initialize();

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

        let mut res_board = Board::new(
            self.board.problem_id,
            self.orig_problem.clone(),
            "upsolve-oka-solver",
            false,
        );

        for i in 0..self.board.musicians().len() {
            res_board.set_volume(i, 10.0);
        }

        let mut remaining_musicians = vec![];
        for i in 0..self.board.musicians().len() {
            if let Some((p, _)) = self.board.musicians()[i] {
                res_board.try_place(i, p.to_point()).unwrap();
            } else {
                remaining_musicians.push(i);
            }
        }

        // Place remaining musicians
        for x in ((res_board.prob.stage.min.x as usize)..=(res_board.prob.stage.max.x as usize))
            .step_by(10)
        {
            if remaining_musicians.is_empty() {
                break;
            }

            for y in ((res_board.prob.stage.min.y as usize)..=(res_board.prob.stage.max.y as usize))
                .step_by(10)
            {
                if remaining_musicians.is_empty() {
                    break;
                }

                let p = P::new(x as f64, y as f64);

                let m = remaining_musicians.last().unwrap();

                let prev_score = res_board.score();

                if res_board.try_place(*m, p.to_point()).is_err() {
                    continue;
                }
                if res_board.score() < prev_score {
                    res_board.unplace(*m);
                    continue;
                }

                remaining_musicians.pop();
            }
        }

        res_board.hungarian();

        res_board
    }

    fn set_visibility(&mut self, m: usize, visible: bool) -> Result<()> {
        if self.is_visible[m] == visible {
            return Ok(());
        }

        if visible {
            self.board.try_place(m, self.musicians[m].to_point())?;
            self.is_visible[m] = true;
            self.visible_musicians_count += 1;
        } else {
            self.board.unplace(m);
            self.is_visible[m] = false;
            self.visible_musicians_count -= 1;
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

        if !self.board.can_place(m, p.to_point()) {
            bail!("Cannot place");
        }

        self.board.unplace(m);

        self.board.try_place(m, p.to_point()).unwrap();

        self.musicians[m] = p;

        Ok(())
    }

    fn temp(&self, iter: usize) -> f64 {
        let max_temp = self.params.max_temp;
        let min_temp = self.params.min_temp;

        let r = iter as f64 / self.num_iter as f64;

        min_temp + (max_temp - min_temp) * (1. - r).powf(self.params.temp_func_power)
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

        debug_assert_eq!(score, self.board.score(), "{:?}", action);
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
            Action::Unplace(x, _) => {
                debug_assert!(self.is_visible[x]);
                self.set_visibility(x, false).is_ok()
            }
            Action::Place(x, p) => {
                debug_assert!(!self.is_visible[x]);
                self.move_musician_to(x, p).unwrap();
                self.set_visibility(x, true).is_ok()
            }
            Action::Swap(x, y) => {
                let x_vis = self.is_visible[x];
                let y_vis = self.is_visible[y];

                let xp = self.musicians[x];
                let yp = self.musicians[y];

                self.board.swap(x, y);

                self.is_visible[x] = y_vis;
                self.is_visible[y] = x_vis;

                self.musicians[x] = yp;
                self.musicians[y] = xp;

                true
            }
            Action::MoveTo(m, _, p) => self.move_musician_to(m, p).is_ok(),
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
        if self.rng.gen_range(0..self.params.hungarian_rarity) == 0 {
            return Action::Hungarian;
        }

        loop {
            let v = self.rng.gen_range(0..80);

            if (0..self.params.v2_unplace).contains(&v) {
                let Some(x) = self.random_visible_musician() else {continue};
                return Action::Unplace(x, self.musicians[x]);
            } else if (20..20 + self.params.v2_place).contains(&v) {
                let Some(x) = self.random_invisible_musician() else {continue};
                let p = self.random_place();

                return Action::Place(x, p);
            } else if (40..40 + self.params.v2_move_dir).contains(&v) {
                let Some(x) = self.random_visible_musician() else {continue};
                let orig = self.musicians[x];

                let dir = self.random_direction(iter);

                let dest = self.move_to_dir(orig, dir);

                if orig == dest {
                    continue;
                }

                return Action::MoveTo(x, orig, dest);
            } else if (60..60 + self.params.v2_swap).contains(&v) {
                loop {
                    let x = self.random_musician();
                    let y = self.random_musician();

                    if self.orig_problem.musicians[x] == self.orig_problem.musicians[y] {
                        continue;
                    }
                    if !self.is_visible[x] && !self.is_visible[y] {
                        continue;
                    }

                    return Action::Swap(x, y);
                }
            }
        }
    }

    fn random_direction(&mut self, iter: usize) -> P {
        let d: f64 = self.rng.gen_range(0.0..1.0);
        let r = self.rng.gen_range(-1.0..1.0) * PI;

        let (x, y) = r.sin_cos();

        let dd = d.powi(2);

        let max_dist = self.params.max_move_dist
            - (self.params.max_move_dist - self.params.min_move_dist)
                * (self.params.max_temp - self.temp(iter))
                / (self.params.max_temp - self.params.min_temp);

        P::new(x * max_dist * dd, y * max_dist * dd)
    }

    fn random_musician(&mut self) -> usize {
        self.rng.gen_range(0..self.board.musicians().len())
    }

    fn random_visible_musician(&mut self) -> Option<usize> {
        if self.visible_musicians_count == 0 {
            return None;
        }
        loop {
            let x = self.rng.gen_range(0..self.board.musicians().len());
            if self.is_visible[x] {
                return x.into();
            }
        }
    }

    fn random_invisible_musician(&mut self) -> Option<usize> {
        if self.visible_musicians_count == self.is_visible.len() {
            return None;
        }
        loop {
            let x = self.rng.gen_range(0..self.board.musicians().len());
            if !self.is_visible[x] {
                return x.into();
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
        if self.forbidden_area.contains(p.to_point()) {
            return p;
        }

        let dest = {
            let dest = p + dir;

            let stage = self.board.prob.stage;

            let x = dest.x.min(stage.max.x).max(stage.min.x);
            let y = dest.y.min(stage.max.y).max(stage.min.y);

            P::new(x, y)
        };

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

            nq + ndir
        } else if let Some(q) = l.vertical_line_intersection(self.forbidden_area.max.x) {
            let dir = dest - q.to_vector();

            let nq = self.translate_point_on_forbideen_bounding_box(q.to_vector());
            let ndir = P::new(dir.y, -dir.x);

            nq + ndir
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
    Unplace(usize, /* orig */ P),
    Place(usize, P),
    Swap(usize, usize),
    MoveTo(usize, /* from */ P, /* to */ P),
    Hungarian,
}

impl Action {
    fn invert(self) -> Action {
        match self {
            Action::Unplace(x, p) => Action::Place(x, p),
            Action::Place(x, p) => Action::Unplace(x, p),
            Action::Swap(x, y) => Action::Swap(x, y),
            Action::MoveTo(m, orig, p) => Action::MoveTo(m, p, orig),
            Action::Hungarian => Action::Hungarian,
        }
    }
}
