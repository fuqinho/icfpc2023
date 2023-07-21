use std::{f64::consts::PI, ops::Range};

use anyhow::{bail, Result};
use log::info;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{params::Params, pretty::pretty};

use super::{
    etx_problem::ExtProblem,
    types::{Board, P},
};

pub struct MiniSolver {
    board: Board,

    num_iter: usize,
    iter_range: Range<usize>,

    rng: SmallRng,

    visible_musicians_count: usize,
    is_visible: Vec<bool>,
    musicians: Vec<P>,

    params: Params,

    available_musicians: Vec<usize>,
    initial_locations: Option<Vec<P>>,

    show_progress: bool,
}

impl MiniSolver {
    pub fn new(
        problem_id: u32,
        problem: ExtProblem,
        num_iter: usize,
        iter_range: Range<usize>,
        params: Params,
        available_musicians: Vec<usize>,
        initial_locations: Option<Vec<P>>,
        seed: u64,
        show_progress: bool,
    ) -> Self {
        let board = Board::new_with_options(
            problem_id,
            problem.problem,
            format!("upsolver-oka-solver3-{seed}"),
            false,
            problem.walls,
            Default::default(),
        );

        let is_visible = vec![false; board.musicians().len()];
        let musicians = vec![board.prob.stage.min.to_vector(); board.musicians().len()];

        Self {
            board,
            num_iter,
            iter_range,
            rng: SmallRng::seed_from_u64(seed),
            visible_musicians_count: 0,
            is_visible,
            musicians,
            params,
            available_musicians,
            initial_locations,
            show_progress,
        }
    }

    pub fn initialize(&mut self) {
        for i in 0..self.board.musicians().len() {
            self.board.set_volume(i, 10.0);
        }

        let initial_visible_musicians_count =
            (self.available_musicians.len() as f64 * self.params.placed_musicians_ratio) as usize;

        if let Some(initial_locations) = self.initial_locations.clone() {
            for (i, p) in initial_locations.into_iter().enumerate() {
                self.move_musician_to(self.available_musicians[i], p)
                    .unwrap();

                self.set_visibility(self.available_musicians[i], true)
                    .unwrap();
            }
            return;
        }

        // Initialize with random positions
        for i in 0..initial_visible_musicians_count {
            let m = self.available_musicians[i];
            loop {
                let p = self.random_place();

                self.move_musician_to(m, p).unwrap();

                if self.set_visibility(m, true).is_ok() {
                    break;
                }
            }
        }
    }

    pub fn solve(&mut self) -> Board {
        assert!(self.available_musicians.len() > 0);

        self.initialize();

        for iter in self.iter_range.clone() {
            self.step(iter);

            if self.show_progress && iter % (self.num_iter / 100) == 0 {
                let temp = self.temp(iter);
                info!(
                    "{} {:>3}% iter: {:>10}  score: {:>14}  temp: {:>10}",
                    self.board.solver,
                    (iter * 100) / self.num_iter,
                    pretty(iter as i64),
                    pretty(self.board.score() as i64),
                    pretty(temp as i64),
                );
            }
        }

        let mut remaining_musicians = vec![];
        for m in self.available_musicians.iter() {
            if self.board.musicians()[*m].is_none() {
                remaining_musicians.push(*m);
            }
        }

        // Place remaining musicians
        'outer: for accept_decrease in (0..u64::MAX).step_by(10000) {
            for x in ((self.board.prob.stage.min.x.ceil() as usize)
                ..=(self.board.prob.stage.max.x as usize))
                .step_by(10)
            {
                if remaining_musicians.is_empty() {
                    break 'outer;
                }

                for y in ((self.board.prob.stage.min.y.ceil() as usize)
                    ..=(self.board.prob.stage.max.y as usize))
                    .step_by(10)
                {
                    if remaining_musicians.is_empty() {
                        break 'outer;
                    }

                    let p = P::new(x as f64, y as f64);

                    let m = *remaining_musicians.last().unwrap();

                    let prev_score = self.board.score();

                    if self.board.try_place(m, p.to_point()).is_err() {
                        continue;
                    }
                    if self.board.score() < prev_score + accept_decrease as f64 {
                        self.board.unplace(m);
                        continue;
                    }

                    remaining_musicians.pop();
                }
            }
        }

        debug_assert_eq!(remaining_musicians.len(), 0);

        self.board.clone()
    }

    fn set_visibility(&mut self, m: usize, visible: bool) -> Result<()> {
        debug_assert!(self.available_musicians.contains(&m));

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
        debug_assert!(self.available_musicians.contains(&m));

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
        }
    }

    fn random_action(&mut self, iter: usize) -> Action {
        loop {
            let v = self.rng.gen_range(0..60);

            if (0..self.params.swap).contains(&v) {
                loop {
                    let x = self.random_musician();
                    let y = self.random_musician();

                    if self.board.prob.musicians[x] == self.board.prob.musicians[y] {
                        continue;
                    }
                    if !self.is_visible[x] && !self.is_visible[y] {
                        continue;
                    }

                    return Action::Swap(x, y);
                }
            } else if (20..(20 + self.params.move_random)).contains(&v) {
                let Some(x) = self.random_visible_musician() else {continue};
                let orig = self.musicians[x];
                let p = self.random_place();

                return Action::MoveTo(x, orig, p);
            } else if (40..(40 + self.params.move_dir)).contains(&v) {
                let Some(x) = self.random_visible_musician() else {continue};
                let orig = self.musicians[x];

                let dir = self.random_direction(iter);

                let dest = orig + dir;

                if orig == dest {
                    continue;
                }

                return Action::MoveTo(x, orig, dest);
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
        self.available_musicians[self.rng.gen_range(0..self.available_musicians.len())]
    }

    fn random_visible_musician(&mut self) -> Option<usize> {
        if self.visible_musicians_count == 0 {
            return None;
        }
        loop {
            let x = self.random_musician();
            if self.is_visible[x] {
                return x.into();
            }
        }
    }

    fn random_place(&mut self) -> P {
        let x = self
            .rng
            .gen_range(self.board.prob.stage.min.x..self.board.prob.stage.max.x);
        let y = self
            .rng
            .gen_range(self.board.prob.stage.min.y..self.board.prob.stage.max.y);

        let p = P::new(x, y);

        p
    }
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Swap(usize, usize),
    MoveTo(usize, /* from */ P, /* to */ P),
}

impl Action {
    fn invert(&self) -> Action {
        match self {
            Action::Swap(x, y) => Action::Swap(*x, *y),
            Action::MoveTo(m, orig, p) => Action::MoveTo(*m, *p, *orig),
        }
    }
}
