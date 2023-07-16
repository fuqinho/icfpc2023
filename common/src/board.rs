use std::f64::consts::PI;

use anyhow::bail;
use euclid::{Box2D, Vector2D};
use lyon_geom::{LineSegment, Point};
use pathfinding::{kuhn_munkres::kuhn_munkres, prelude::Matrix};

use crate::{
    board_options::BoardOptions,
    float::{Float, F32, F64},
    geom::tangent_to_circle,
    vec2::Vec2,
    Placement, Problem, Solution,
};

use anyhow::Result;

type P = Vector2D<f64, euclid::UnknownUnit>;

const MUSICIAN_R: f64 = 5.;

#[derive(Clone, Debug)]
pub struct Board<F: Float = F64> {
    pub problem_id: u32,
    pub solver: String,
    // NB: stage is modified
    pub prob: Problem,

    // m -> position
    // musicians + pillars
    ps: Vec<Option<(P, f64)>>,
    // m -> important audience ids sorted by args
    aids: Vec2<(F, u32)>,

    // m -> a -> j s.t. aids[j].1 == a
    aids_rev: Vec2<Option<usize>>,

    // m -> j -> block count of aids[j].1
    // i.e. number of important musicians or pillars between m and aids[j].1.
    blocks: Vec2<u8>,

    // m -> closeness factor
    qs: Vec<f64>,
    // m -> I
    impacts: Vec<f64>,

    // m -> j -> impact between m and aids[j].1.
    individual_impacts: Vec2<i64>,

    // m -> a -> visibility [0., 1.] (0. == completely invisible)
    visibility: Vec<Vec<f64>>,
    use_visibility: bool,

    // ins -> an available musician
    available_musician: Vec<Option<usize>>,

    volumes: Vec<f64>,

    options: BoardOptions,
}

impl Board<F64> {
    pub fn new<T: AsRef<str>>(
        problem_id: u32,
        prob: Problem,
        solver: T,
        use_visibility: bool,
    ) -> Self {
        Self::new_with_options(problem_id, prob, solver, use_visibility, Default::default())
    }
}

impl Board<F32> {
    pub fn new_f32<T: AsRef<str>>(
        problem_id: u32,
        prob: Problem,
        solver: T,
        use_visibility: bool,
    ) -> Self {
        Self::new_with_options(problem_id, prob, solver, use_visibility, Default::default())
    }
}

impl<F: Float> Board<F> {
    pub fn new_with_options<T: AsRef<str>>(
        problem_id: u32,
        mut prob: Problem,
        solver: T,
        use_visibility: bool,
        options: BoardOptions,
    ) -> Self {
        let n = prob.musicians.len();
        let m = prob.attendees.len();
        let p = prob.pillars.len();

        let mut ps = vec![None; n + p];
        for i in 0..p {
            ps[i + n] = Some((prob.pillars[i].center.to_vector(), prob.pillars[i].radius));
        }

        let important_attendees_count =
            (prob.attendees.len() as f64 * options.important_attendees_ratio).ceil() as usize;

        let aids = Vec2::new(n, important_attendees_count, (F::new(0.), 0));
        let aids_rev = Vec2::new(n, m, None);

        let blocks = Vec2::new(n, important_attendees_count, 0);
        let qs = vec![1.; n];
        let impacts = vec![0.; n];
        let individual_impacts = Vec2::new(n, important_attendees_count, 0);
        let visibility = vec![vec![1.; m]; n];

        let mut available_musician = vec![None; prob.attendees[0].tastes.len()];
        for (m, i) in prob.musicians.iter().enumerate() {
            available_musician[*i] = Some(m);
        }

        prob.stage = Box2D::new(
            prob.stage.min + P::new(10., 10.),
            prob.stage.max - P::new(10., 10.),
        );

        Self {
            problem_id,
            solver: solver.as_ref().to_owned(),
            prob,
            ps,
            aids,
            aids_rev,
            blocks,
            qs,
            impacts,
            individual_impacts,
            visibility,
            use_visibility,
            available_musician,
            volumes: vec![1.; n],
            options,
        }
    }

    pub fn score(&self) -> f64 {
        let mut res = 0.;
        for m in 0..self.musicians().len() {
            res += (self.volumes[m] * self.qs[m] * self.impacts[m]).ceil();
        }
        res
    }

    pub fn score_ignore_negative(&self) -> f64 {
        let mut res = 0.;
        for m in 0..self.musicians().len() {
            res += (self.volumes[m] * self.qs[m] * self.impacts[m])
                .max(0.0)
                .ceil();
        }
        res
    }

    pub fn musicians(&self) -> &[Option<(P, f64)>] {
        &self.ps[0..self.prob.musicians.len()]
    }

    pub fn volume(&self, m: usize) -> f64 {
        self.volumes[m]
    }

    pub fn set_volume(&mut self, m: usize, volume: f64) {
        self.volumes[m] = volume;
    }

    pub fn q(&self, m: usize) -> f64 {
        self.qs[m]
    }

    // The musician's contribution to the score
    pub fn contribution(&self, m: usize) -> f64 {
        let mut res = 0;
        for j in 0..self.aids.len2() {
            if *self.blocks.get(m, j) > 0 {
                continue;
            }
            res += self.individual_impacts.get(m, j);
        }
        res as f64
    }

    pub fn contribution2(&self, m: usize) -> f64 {
        // (self.volumes[m] * self.qs[m] * self.impacts[m]).ceil()

        (self.qs[m] * self.impacts[m]).ceil()
    }

    pub fn contribution_if_instrument(&self, m: usize, ins: usize) -> f64 {
        let mut res = 0.;

        for j in 0..self.aids.len2() {
            if *self.blocks.get(m, j) > 0 {
                continue;
            }
            res += self.impact_if_kind(m, j, ins);
        }
        res
    }

    pub fn contribution_for(&self, m: usize, a: usize) -> f64 {
        let Some(j) = self.aids_rev.get(m, a) else {return 0.0};

        if *self.blocks.get(m, *j) > 0 {
            return 0.;
        }
        *self.individual_impacts.get(m, *j) as f64
    }

    pub fn score_increase_if_put_musician_on(&mut self, m: usize, p: Point<f64>) -> Result<f64> {
        let prev_score = self.score();

        self.try_place(m, p)?;

        let res = self.score() - prev_score;

        self.unplace(m);

        Ok(res)
    }

    pub fn score_increase_if_put_instrument_on(
        &mut self,
        ins: usize,
        p: Point<f64>,
    ) -> Result<f64> {
        let m = self
            .available_musician_with_instrument(ins)
            .ok_or_else(|| anyhow::anyhow!("no musician with instrument {} available", ins))?;
        self.score_increase_if_put_musician_on(m, p)
    }

    pub fn available_musician_with_instrument(&mut self, ins: usize) -> Option<usize> {
        return self.available_musician[ins];
    }

    pub fn is_musician_seeing(&self, m: usize, a: usize) -> bool {
        let Some(j) = self.aids_rev.get(m,a) else { return false };
        *self.blocks.get(m, *j) == 0
    }

    pub fn try_place(&mut self, i: usize, position: Point<f64>) -> anyhow::Result<()> {
        let mut bb = self.prob.stage;
        bb.max += P::new(1e-9, 1e-9);
        if !bb.contains(position) {
            bail!("not on stage {:?} {:?}", position, self.prob.stage);
        }
        for p in self.ps[0..self.prob.musicians.len()].iter() {
            if let Some((p, _)) = p {
                if (*p - position.to_vector()).square_length() < 100. {
                    bail!("too close to another musician {:?}: {:?}", p, position);
                }
            }
        }
        if self.ps[i].is_some() {
            bail!("already placed");
        }

        Ok(self.place(i, position.to_vector()))
    }

    fn place(&mut self, m: usize, p: P) {
        // Update ps and impacts
        self.ps[m] = Some((p, MUSICIAN_R));

        // Update qs
        self.update_qs(m, true);

        // Update aids
        let mut max_dist2 = None;
        if self.options.important_attendees_ratio < 1.0 {
            let mut dists = self
                .prob
                .attendees
                .iter()
                .map(|a| (a.position - p).to_vector().square_length().into())
                .collect::<Vec<F64>>();
            dists.sort_unstable();
            max_dist2 = Some(dists[self.aids.len2() - 1].get());
        }

        let mut cnt = 0;
        for (i, a) in self.prob.attendees.iter().enumerate() {
            let is_important = if let Some(max_dist2) = max_dist2 {
                let dist = (a.position - p).to_vector().square_length();
                dist <= max_dist2
            } else {
                true
            };

            if is_important && cnt < self.aids.len2() {
                let r: f64 = (a.position - p).to_vector().angle_from_x_axis().radians;
                self.aids.set(m, cnt, (r.into(), i as u32));
                cnt += 1;
            }
        }
        debug_assert_eq!(cnt, self.aids.len2());

        self.aids.row_mut(m).sort_unstable();

        self.aids_rev.row_mut(m).fill(None);
        for j in 0..self.aids.len2() {
            self.aids_rev
                .set(m, self.aids.get(m, j).1 as usize, j.into());
        }

        self.impacts[m] = 0.0;
        for j in 0..self.aids.len2() {
            self.individual_impacts.set(m, j, self.impact(m, j) as i64);
            self.impacts[m] += *self.individual_impacts.get(m, j) as f64;
        }

        // Update blocks
        self.update_blocks(m, p, true);

        self.update_available_musician(m);
    }

    // Returns whether i can be moved to the position.
    pub fn can_place(&self, i: usize, position: Point<f64>) -> bool {
        let mut bb = self.prob.stage;
        bb.max += P::new(1e-9, 1e-9);
        if !bb.contains(position) {
            return false;
        }
        for (ix, p) in self.ps[0..self.prob.musicians.len()].iter().enumerate() {
            if ix == i {
                continue;
            }
            if let Some((p, _)) = p {
                if (*p - position.to_vector()).square_length() < 100. {
                    return false;
                }
            }
        }
        true
    }

    pub fn unplace(&mut self, m: usize) {
        let (p, _) = self.ps[m].unwrap();

        // Update blocks
        self.update_blocks(m, p, false);

        // Update qs
        self.update_qs(m, false);

        // Update ps and impacts.
        self.ps[m] = None;
        self.impacts[m] = 0.;

        self.update_available_musician(m);
    }

    fn update_qs(&mut self, m: usize, inc: bool) {
        if !self.prob.is_v2() {
            return;
        }

        let sig = if inc { 1. } else { -1. };

        let p = self.musicians()[m].unwrap().0;

        for i in 0..self.prob.musicians.len() {
            if i == m || self.prob.musicians[i] != self.prob.musicians[m] {
                continue;
            }

            if let Some((q, _)) = self.ps[i] {
                let d = sig / (p - q).length();

                self.qs[m] += d;
                self.qs[i] += d;
            }
        }
    }

    fn vis_f(ratio: f64) -> f64 {
        const THREASHOLD: f64 = 0.0;
        if ratio < THREASHOLD {
            1e-6
        } else {
            (ratio - THREASHOLD) / (1. - THREASHOLD)
        }
    }

    fn update_blocks(&mut self, m: usize, p: P, inc: bool) {
        let (prob, ps, all_aids, blocks, impacts, individual_impacts) = (
            &self.prob,
            &self.ps,
            &self.aids,
            &mut self.blocks,
            &mut self.impacts,
            &self.individual_impacts,
        );

        let mut rs = [None, None];

        let eps = F::EPS;
        let pi_plus_eps = F::new(PI + eps);

        for (i, q) in ps.iter().enumerate() {
            if i == m {
                continue;
            }

            let Some((q, r)) = q else { continue; };

            if self.options.important_musician_range_squared < f64::INFINITY
                && i < prob.musicians.len()
            {
                let dist2 = (p - *q).square_length();
                if dist2 > self.options.important_musician_range_squared {
                    continue;
                }
            }

            let (t1, t2) = tangent_to_circle(p, *q, *r);
            let base_r1 = (t1 - p).angle_from_x_axis().radians;
            let base_r2 = (t2 - p).angle_from_x_axis().radians;

            for m_is_blocked in [false, true] {
                let (blocking, blocked, blocked_i, blocking_i) = if m_is_blocked {
                    (*q, p, m, i)
                } else {
                    if i >= prob.musicians.len() {
                        // blocked is pillar, we don't need to calculate impact
                        continue;
                    }
                    (p, *q, i, m)
                };

                // Update for blocked musician.

                let (r1, r2) = if m_is_blocked {
                    (base_r1 + eps, base_r2 - eps)
                } else {
                    (opposite_angle(base_r1) + eps, opposite_angle(base_r2) - eps)
                };

                let (f_r1, f_r2) = (F::new(r1), F::new(r2));

                let j1 = all_aids.row(blocked_i).partition_point(|r| r.0 < f_r1);

                let rs_len;
                if f_r1 < f_r2 {
                    rs[0] = Some(f_r2);
                    rs_len = 1;
                } else {
                    rs[0] = Some(pi_plus_eps);
                    rs[1] = Some(f_r2);
                    rs_len = 2;
                }

                for ri in 0..rs_len {
                    let j1 = if ri == 0 { j1 } else { 0 };

                    let r2 = rs[ri].unwrap();

                    for j in j1..all_aids.len2() {
                        if all_aids.get(blocked_i, j).0 > r2 {
                            break;
                        }

                        let vis = if self.use_visibility {
                            let r = all_aids.get(blocked_i, j).0;
                            let r1 = if ri == 0 { f_r1 } else { F::new(-PI - eps) };

                            let vis = 1.
                                - (r2.get() - r.get()).min(r.get() - r1.get())
                                    / ((r2.get() - r1.get()) / 2.)
                                + 1e-6;
                            let vis = Self::vis_f(vis);
                            vis
                        } else {
                            0.0
                        };

                        debug_assert!({
                            let a = all_aids.get(blocked_i, j).1 as usize;
                            let distance_to_blocker_sq = (blocked - blocking).square_length();
                            let distance_to_attendee_sq = (self.prob.attendees[a].position
                                - blocked)
                                .to_vector()
                                .square_length();
                            !(distance_to_attendee_sq <= distance_to_blocker_sq
                                && blocking_i < prob.musicians.len())
                        });

                        if blocking_i >= prob.musicians.len() {
                            let a = all_aids.get(blocked_i, j).1 as usize;
                            let distance_to_blocker_sq = (blocked - blocking).square_length();
                            let distance_to_attendee_sq = (self.prob.attendees[a].position
                                - blocked)
                                .to_vector()
                                .square_length();
                            if distance_to_attendee_sq <= distance_to_blocker_sq {
                                continue;
                            }
                        }

                        if inc {
                            let b = blocks.get_mut(blocked_i, j);
                            *b += 1;
                            let impact = *individual_impacts.get(blocked_i, j) as f64;

                            if self.use_visibility {
                                let a = all_aids.get(blocked_i, j).1 as usize;
                                let prev_vis = self.visibility[blocked_i][a];
                                self.visibility[blocked_i][a] *= vis;
                                impacts[blocked_i] +=
                                    (self.visibility[blocked_i][a] - prev_vis) * impact;
                            } else {
                                if *b == 1 {
                                    impacts[blocked_i] -= impact;
                                }
                            }
                        } else {
                            let b = blocks.get_mut(blocked_i, j);
                            *b -= 1;

                            let impact = *individual_impacts.get(blocked_i, j) as f64;
                            if self.use_visibility {
                                let a = all_aids.get(blocked_i, j).1 as usize;
                                let prev_vis = self.visibility[blocked_i][a];
                                self.visibility[blocked_i][a] /= vis;

                                impacts[blocked_i] +=
                                    (self.visibility[blocked_i][a] - prev_vis) * impact;
                            } else {
                                if *b == 0 {
                                    impacts[blocked_i] += impact;
                                }
                            }
                        }
                    }
                }

                // for debug. Please keep it.
                if false {
                    let distance_to_blocker_sq = (blocked - blocking).square_length();

                    let (r1, r2) = (f_r1, f_r2);

                    for (r, a) in all_aids.row(blocked_i).iter() {
                        let a = *a as usize;

                        let seg = LineSegment {
                            from: self.prob.attendees[a].position,
                            to: self.ps[blocked_i].unwrap().0.to_point(),
                        };
                        let included = if r1 < r2 {
                            r1 < *r && *r <= r2
                        } else {
                            *r <= r2 || r1 <= *r
                        } && seg.square_length() > distance_to_blocker_sq;
                        let is_blocked = seg
                            .distance_to_point(self.ps[blocking_i].unwrap().0.to_point())
                            < self.ps[blocking_i].unwrap().1;
                        if included != is_blocked {
                            eprintln!("Blocking mismatch: blocker={} {}({:?}), blocked={}({:?}), attendee={}({:?}), distance={:?}, radius={:?}, angle={:?}, angle_range=({:?}, {:?}, t1={:?}, t2={:?})",
                                if blocking_i < self.prob.musicians.len() { "musician" } else { "pillar" },
                                if blocking_i < self.prob.musicians.len() { blocking_i } else { blocking_i - self.prob.musicians.len() },
                                self.ps[blocking_i].unwrap().0.to_point(),
                                blocked_i, self.ps[blocked_i].unwrap().0.to_point(),
                                a, self.prob.attendees[a].position,
                                seg.distance_to_point(self.ps[blocking_i].unwrap().0.to_point()),
                                self.ps[blocking_i].unwrap().1, *r,
                                r1, r2, t1, t2);
                        }
                    }
                }
            }
        }
    }

    fn update_available_musician(&mut self, m: usize) {
        let ins = self.prob.musicians[m];

        let m_is_available = self.ps[m].is_none();

        if m_is_available {
            self.available_musician[ins] = Some(m);
            return;
        }

        if let Some(m2) = self.available_musician[ins] {
            if m2 != m {
                return;
            }
        }

        self.available_musician[ins] = None;

        for (m, p) in self.musicians().iter().enumerate() {
            if p.is_some() {
                continue;
            }
            if self.prob.musicians[m] == ins {
                self.available_musician[ins] = Some(m);
                return;
            }
        }
    }

    #[inline]
    fn impact(&self, m: usize, j: usize) -> f64 {
        let k = self.prob.musicians[m];
        self.impact_if_kind(m, j, k)
    }

    #[inline]
    fn impact_if_kind(&self, m: usize, j: usize, k: usize) -> f64 {
        let d2 = (self.prob.attendees[self.aids.get(m, j).1 as usize].position
            - self.ps[m].unwrap().0)
            .to_vector()
            .square_length();

        let impact =
            1_000_000.0 * self.prob.attendees[self.aids.get(m, j).1 as usize].tastes[k] / d2;
        impact.ceil()
    }

    pub fn solution(&self) -> Result<Solution> {
        let mut placements = vec![];
        for p in self.ps[0..self.prob.musicians.len()].iter() {
            if let Some((p, _)) = p {
                placements.push(Placement {
                    position: p.to_point(),
                });
            } else {
                bail!("not all musicians are placed");
            }
        }
        Ok(Solution {
            problem_id: self.problem_id,
            solver: self.solver.clone(),
            placements,
            volumes: self.volumes.clone(),
        })
    }

    pub fn solution_with_optimized_volume(&self) -> Result<Solution> {
        let mut sol = self.solution()?;
        for m in 0..sol.volumes.len() {
            if self.contribution(m) < 0.0 {
                sol.volumes[m] = 0.0;
            } else {
                sol.volumes[m] = 10.0;
            }
        }
        Ok(sol)
    }

    // Optimize musician allocations with hungarian algorithm in O(|musicians|^3).
    pub fn hungarian(&mut self) {
        let mut weights = vec![vec![0; self.prob.musicians.len()]; self.prob.musicians.len()];

        for m in 0..self.musicians().len() {
            for m2 in 0..self.musicians().len() {
                if self.musicians()[m2].is_none() {
                    continue;
                }

                weights[m][m2] = self
                    .contribution_if_instrument(m2, self.prob.musicians[m])
                    .max(0.) as i64;
            }
        }

        let matrix = Matrix::from_rows(weights).unwrap();

        let (_, assignment) = kuhn_munkres(&matrix);

        let ps: Vec<_> = self
            .musicians()
            .iter()
            .map(|p| p.map(|(p, _)| p.to_point()))
            .collect();

        for m in 0..ps.len() {
            if ps[m].is_some() {
                self.unplace(m);
            }
        }

        for (m, m2) in assignment.into_iter().enumerate() {
            if let Some(p) = ps[m2] {
                self.try_place(m, p).unwrap();
            }
        }
    }

    pub fn swap(&mut self, m: usize, m2: usize) {
        if m == m2 {
            return;
        }
        if self.ps[m].is_none() && self.ps[m2].is_none() {
            return;
        }
        if self.prob.is_v2() {
            panic!("cannot swap musicians in v2");
        }
        if self.use_visibility {
            panic!("cannot swap musicians if use_visibility is set");
        }

        // Swap ps, aids, aids_rev, volumes, blocks
        self.ps.swap(m, m2);
        for i in 0..self.aids.len2() {
            self.aids.swap(m, i, m2, i);
        }
        for i in 0..self.aids_rev.len2() {
            self.aids_rev.swap(m, i, m2, i);
        }
        self.volumes.swap(m, m2);
        for i in 0..self.blocks.len2() {
            self.blocks.swap(m, i, m2, i);
        }

        // Recompute individual_impacts and impacts.
        for i in [m, m2] {
            self.impacts[i] = 0.;

            if self.ps[i].is_some() {
                for j in 0..self.aids.len2() {
                    let impact = self.impact(i, j);
                    self.individual_impacts.set(i, j, impact as i64);
                    if *self.blocks.get(i, j) == 0 {
                        self.impacts[i] += impact;
                    }
                }
            }

            self.update_available_musician(i);
        }
    }
}

fn opposite_angle(mut r: f64) -> f64 {
    r += PI;
    if r > PI {
        r -= 2. * PI;
    }
    r
}

impl TryInto<Solution> for Board {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Solution, Self::Error> {
        self.solution()
    }
}

#[cfg(test)]
mod tests {
    use lyon_geom::{Box2D, Point};
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::{board::Board, board_options::BoardOptions, evaluate, Attendee, Problem, Solution};

    #[test]
    fn test_board() {
        for (important_attendees_ratio, expected) in [(1.0, None), (0.99, Some(434649.0))] {
            let problem_id = 42u32;

            let mut rng = StdRng::seed_from_u64(42);

            let problem = Problem::read_from_file(format!("../problems/{}.json", 42)).unwrap();

            let options =
                BoardOptions::default().with_important_attendees_ratio(important_attendees_ratio);
            let mut board =
                Board::new_with_options(problem_id, problem.clone(), "test_solver", false, options);

            for i in 0..board.prob.musicians.len() {
                loop {
                    let x: f64 = rng.gen_range(board.prob.stage.min.x..board.prob.stage.max.x);
                    let y: f64 = rng.gen_range(board.prob.stage.min.y..board.prob.stage.max.y);
                    if board.try_place(i, Point::new(x, y)).is_ok() {
                        break;
                    }
                }
            }

            let solution: Solution = board.clone().try_into().unwrap();

            let expected_score = expected.unwrap_or_else(|| evaluate(&problem, &solution));

            assert_eq!(
                board.score(),
                expected_score,
                "failed on important_attendees_ratio {}",
                important_attendees_ratio
            );

            for i in 0..problem.musicians.len() {
                board.unplace(i);
            }

            assert_eq!(board.score(), 0.0);
        }
    }

    #[test]
    fn test_tangent() {
        for flip in [false, true] {
            let pnt = |x: f64, y: f64| {
                if flip {
                    Point::new(y, x)
                } else {
                    Point::new(x, y)
                }
            };

            let problem = Problem {
                room: Box2D::new(Point::new(0.0, 0.0), Point::new(1000.0, 1000.0)),
                stage: Box2D::new(Point::new(0.0, 0.0), Point::new(100.0, 100.0)),
                musicians: vec![0, 0, 1],
                attendees: vec![Attendee {
                    position: pnt(110., 15.),
                    tastes: vec![0.0, 1.0],
                }],
                pillars: vec![],
            };

            let mut board = Board::new(0, problem.clone(), "test_solver", false);

            board.try_place(0, pnt(20.0, 10.0)).unwrap();
            board.try_place(1, pnt(20.0, 20.0)).unwrap();
            board.try_place(2, pnt(10.0, 15.0)).unwrap();

            let score = board.score();
            let expected_score = 1e6 / 100.0 / 100.0;

            assert_eq!(score, expected_score);

            for eps in [1e-9, -1e-9] {
                board.unplace(2);
                board.try_place(2, pnt(10.0, 15.0 + eps)).unwrap();

                assert_eq!(board.score(), 0.);
            }
        }
    }
}
