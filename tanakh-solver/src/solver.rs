use common::{board::Board, Attendee, Pillar, Problem, Solution};
use euclid::{default::*, point2, vec2};
use lyon_geom::{LineSegment, Point};
use rand::Rng;

const SOLVER_NAME: &str = "(´･_･`)v3";

pub struct Solver2<'a> {
    pub problem_id: u32,
    pub problem: &'a common::Problem,
    pub start_temp: Option<f64>,
    pub better_initial: bool,
    pub initial_solution: Option<&'a common::Solution>,
    pub taste: Option<usize>,
    pub param: String,
}

pub struct State2 {
    board: Board,
}

impl saru::State for State2 {
    type Solution = common::Solution;

    fn solution(&self) -> Self::Solution {
        self.board.solution().unwrap()
    }
}

pub enum Move {
    ChangePos {
        id: usize,
        new_pos: Point2D<f64>,
        old_pos: Point2D<f64>,
    },
    ChangeVolume {
        id: usize,
        new_volume: f64,
        old_volume: f64,
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
    fn gen_change_pos(
        rng: &mut impl Rng,
        board: &Board,
        taste: Option<usize>,
        progress_ratio: f64,
    ) -> Self {
        let stage = &board.prob.stage;

        let scale_x = (stage.width() / 5.0 * (1.0 - progress_ratio)).max(5.0);
        let scale_y = (stage.height() / 5.0 * (1.0 - progress_ratio)).max(5.0);

        let grid_level = (progress_ratio * 8.0).floor() as i32;
        let grid = 1.0 / 2.0_f64.powi(grid_level);
        let scale_x = (scale_x / grid).round() as i32;
        let scale_y = (scale_y / grid).round() as i32;

        'outer: loop {
            let id = rng.gen_range(0..board.musicians().len());

            if let Some(taste) = taste {
                if taste != board.prob.musicians[id] {
                    continue;
                }
            }

            // let theta = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
            // let len = rng.gen_range(0.1..=scale);
            // let d = Vector2D::from_angle_and_length(Angle::radians(theta), len);

            let mut xd = rng.gen_range(-scale_x..=scale_x);
            let mut yd = rng.gen_range(-scale_y..=scale_y);

            let old_pos = board.musicians()[id].unwrap().0.to_point();

            while xd != 0 || yd != 0 {
                let d = vec2(xd as f64 * grid, yd as f64 * grid);

                let new_pos = old_pos + d;
                let new_pos = point2(
                    new_pos.x.clamp(stage.min.x, stage.max.x),
                    new_pos.y.clamp(stage.min.y, stage.max.y),
                );

                if new_pos == old_pos {
                    break;
                }

                if !board.can_place(id, new_pos) {
                    xd /= 2;
                    yd /= 2;
                    continue;
                }

                break 'outer Move::ChangePos {
                    id,
                    new_pos,
                    old_pos,
                };
            }
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

    fn gen_change_volume(rng: &mut impl Rng, board: &Board) -> Self {
        loop {
            let id = rng.gen_range(0..board.prob.musicians.len());
            let old_volume = board.volume(id);
            let new_volume = (old_volume + if rng.gen() { 1.0 } else { -1.0 }).clamp(0.0, 10.0);
            if old_volume != new_volume {
                break Move::ChangeVolume {
                    id,
                    new_volume,
                    old_volume,
                };
            }
        }
    }
}

impl saru::Annealer for Solver2<'_> {
    type State = State2;

    type Move = Move;

    fn init_state(&self, rng: &mut impl Rng) -> Self::State {
        let solver_name = format!("{SOLVER_NAME} ({})", self.param);
        let mut board = Board::new(self.problem_id, self.problem.clone(), &solver_name);

        if let Some(initial_solution) = &self.initial_solution {
            for (i, v) in initial_solution.volumes.iter().enumerate() {
                board.set_volume(i, *v);
            }
            for (i, p) in initial_solution.placements.iter().enumerate() {
                board.try_place(i, p.position).unwrap();
            }
        } else if self.better_initial {
            let g = board.prob.stage.center();
            let mut ix = (0..board.prob.musicians.len())
                .map(|i| {
                    let taste = board.prob.musicians[i];
                    let mut exp = 0.0_f64;
                    for att in &board.prob.attendees {
                        let d2 = (att.position - g).square_length();
                        exp += att.tastes[taste] / d2;
                    }
                    (i, exp)
                })
                .collect::<Vec<_>>();
            ix.sort_by(|(_, e1), (_, e2)| e2.total_cmp(e1));

            for (i, _) in ix {
                board.set_volume(i, 5.5);

                let mut best = (f64::MIN, Point::new(0.0, 0.0));
                for _ in 0..200 {
                    loop {
                        let x: f64 = rng
                            .gen_range(board.prob.stage.min.x..=board.prob.stage.max.x)
                            .round();
                        let y: f64 = rng
                            .gen_range(board.prob.stage.min.y..=board.prob.stage.max.y)
                            .round();
                        if board.try_place(i, Point::new(x, y)).is_ok() {
                            let score = board.score();
                            if score > best.0 {
                                best = (score, Point::new(x, y));
                            }
                            board.unplace(i);
                            break;
                        }
                    }
                }

                board.try_place(i, best.1).unwrap();
            }
        } else {
            for i in 0..board.prob.musicians.len() {
                board.set_volume(i, 5.5);
                loop {
                    let x: f64 = rng
                        .gen_range(board.prob.stage.min.x..=board.prob.stage.max.x)
                        .round();
                    let y: f64 = rng
                        .gen_range(board.prob.stage.min.y..=board.prob.stage.max.y)
                        .round();
                    if board.try_place(i, Point::new(x, y)).is_ok() {
                        break;
                    }
                }
            }
        }

        State2 { board }
    }

    fn start_temp(&self, init_score: f64) -> f64 {
        if let Some(start_temp) = self.start_temp {
            start_temp
        } else {
            (init_score.abs() * 0.1).max(1e6)
        }
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
        loop {
            match rng.gen_range(0..=5) {
                0..=2 => {
                    return Move::gen_change_pos(rng, &state.board, self.taste, progress_ratio)
                }

                3 => loop {
                    let m1 = Move::gen_change_pos(rng, &state.board, self.taste, progress_ratio);
                    let m2 = Move::gen_change_pos(rng, &state.board, self.taste, progress_ratio);

                    match (&m1, &m2) {
                        (
                            Move::ChangePos {
                                id: id1,
                                new_pos: new_pos1,
                                ..
                            },
                            Move::ChangePos {
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

                    return Move::Multiple {
                        moves: vec![m1, m2],
                    };
                },

                4 => {
                    if self.taste.is_some() {
                        continue;
                    } else {
                        return Move::gen_swap(rng, &state.board);
                    }
                }

                // 5 => {
                //     let s1 = Move::gen_swap(rng, &state.board);
                //     let s2 = Move::gen_swap(rng, &state.board);
                //     Move::Multiple {
                //         moves: vec![s1, s2],
                //     }
                // }
                5 => return Move::gen_change_volume(rng, &state.board),

                _ => unreachable!(),
            }
        }
    }

    fn apply(&self, state: &mut Self::State, mov: &Self::Move) {
        match mov {
            Move::ChangePos { id, new_pos, .. } => {
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
            Move::ChangeVolume { id, new_volume, .. } => {
                state.board.set_volume(*id, *new_volume);
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
            Move::ChangePos { id, old_pos, .. } => {
                state.board.unplace(*id);
                state.board.try_place(*id, *old_pos).unwrap();
            }
            Move::Swap { .. } => {
                self.apply(state, mov);
            }
            Move::ChangeVolume { id, old_volume, .. } => {
                state.board.set_volume(*id, *old_volume);
            }
            Move::Multiple { moves } => {
                for mov in moves.iter().rev() {
                    self.unapply(state, mov);
                }
            }
        }
    }
}

fn can_view_stage(stage: &Box2D<f64>, pillars: &[Pillar], attendee: &Attendee) -> bool {
    let v_to_stage = stage.center() - attendee.position;
    let mut stage_angle_min = f64::MAX;
    let mut stage_angle_max = f64::MIN;

    for p in [
        point2(stage.min.x, stage.min.y),
        point2(stage.min.x, stage.max.y),
        point2(stage.max.x, stage.min.y),
        point2(stage.max.x, stage.max.y),
    ] {
        let p = p - attendee.position;
        let angle = v_to_stage.angle_to(p).radians;
        stage_angle_max = stage_angle_max.max(angle);
        stage_angle_min = stage_angle_min.min(angle);
    }

    let mut pillar_angle = vec![];

    for pillar in pillars {
        let pos = pillar.center - attendee.position;
        let angle = v_to_stage.angle_to(pos).radians;
        let d = pos.length();
        let t = (d.powi(2) - pillar.radius.powi(2)).sqrt();
        let theta = pillar.radius.atan2(t);
        assert!(theta > 0.0);
        pillar_angle.push((angle - theta, angle + theta));
    }

    pillar_angle.sort_by(|a, b| a.0.total_cmp(&b.0));

    for (l, r) in pillar_angle {
        if l > stage_angle_min {
            break;
        }
        stage_angle_min = stage_angle_min.max(r);
    }

    stage_angle_max > stage_angle_min
}

fn remove_invisible_atendees(p: &mut Problem) {
    if p.pillars.is_empty() {
        return;
    }

    let mut new_attendees = vec![];

    for attendee in p.attendees.iter() {
        if can_view_stage(&p.stage, &p.pillars, attendee) {
            new_attendees.push(attendee.clone());
        }
    }

    eprintln!(
        "Pruned invisible attendees: {} -> {}",
        p.attendees.len(),
        new_attendees.len()
    );

    p.attendees = new_attendees;
}

pub fn pre_process(p: &mut Problem, prune_far: Option<f64>) {
    eprintln!("Pre-processing...");
    remove_invisible_atendees(p);

    if let Some(prune_dist) = prune_far {
        let p00 = Point::new(p.stage.min.x, p.stage.min.y);
        let p01 = Point::new(p.stage.min.x, p.stage.max.y);
        let p10 = Point::new(p.stage.max.x, p.stage.min.y);
        let p11 = Point::new(p.stage.max.x, p.stage.max.y);

        let s1 = LineSegment { from: p00, to: p01 };
        let s2 = LineSegment { from: p01, to: p11 };
        let s3 = LineSegment { from: p11, to: p10 };
        let s4 = LineSegment { from: p10, to: p00 };

        let ss = vec![s1, s2, s3, s4];

        let mut pruned = vec![];

        for attendee in &mut p.attendees {
            if ss
                .iter()
                .any(|s| s.distance_to_point(attendee.position) < prune_dist)
            {
                pruned.push(attendee.clone());
            }
        }

        eprintln!("Prune attendee : {} -> {}", p.attendees.len(), pruned.len());

        p.attendees = pruned;
    }
}

pub fn post_process(problem_id: u32, p: &Problem, s: &mut Solution) {
    let mut board = Board::new(problem_id, p.clone(), SOLVER_NAME);

    eprintln!("Post processing...");

    for i in 0..s.placements.len() {
        board.try_place(i, s.placements[i].position).unwrap();
        board.set_volume(i, s.volumes[i]);
    }

    let init_score = board.score();
    let init_score_acc = common::evaluate(p, s);

    for iter in 1.. {
        let iter_score = board.score();

        let mut changed = false;
        for i in 0..s.placements.len() {
            let mut pos = s.placements[i].position;
            let mut score = board.score();

            loop {
                let mut local_changed = false;

                for d in [
                    vec2(1.0, 0.0),
                    vec2(-1.0, 0.0),
                    vec2(0.0, 1.0),
                    vec2(0.0, -1.0),
                ] {
                    let new_pos = pos + d;
                    if board.can_place(i, new_pos) {
                        board.unplace(i);
                        board.try_place(i, new_pos).unwrap();

                        let new_score = board.score();
                        if new_score > score {
                            score = new_score;
                            pos = new_pos;
                            local_changed = true;
                        } else {
                            board.unplace(i);
                            board.try_place(i, pos).unwrap();
                        }
                    }
                }

                if local_changed {
                    changed = true;
                } else {
                    break;
                }
            }

            s.placements[i].position = pos;
        }

        if !changed {
            break;
        } else {
            eprintln!(
                "Post processing iter {iter}: {iter_score} -> {} ({:+.3}%)",
                board.score(),
                (board.score() - iter_score) / iter_score * 100.0,
            );
        }
    }

    let post_score = board.score();
    let post_score_acc = common::evaluate(p, s);
    let final_solution = common::evaluate::fixup_volumes(p, s);
    let final_score = board.score();
    let final_score_acc = common::evaluate::evaluate(p, &final_solution);

    eprintln!("Post process, fixup: {post_score} -> {final_score}");

    assert!(
        final_score_acc >= post_score_acc,
        "Fix volume reduce score!!!"
    );

    *s = final_solution;

    eprintln!(
        "Post processed score: {init_score_acc} -> {final_score_acc} ({:+.3}%)",
        (final_score_acc - init_score_acc) / init_score * 100.0,
    );
}
