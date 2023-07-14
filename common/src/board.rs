use std::f64::consts::PI;

use anyhow::bail;
use euclid::{Box2D, Vector2D};
use lyon_geom::{LineSegment, Point};

use crate::{
    board_options::BoardOptions, f64::F64, geom::tangent_to_circle, Placement, Problem, Solution,
};

use anyhow::Result;

type P = Vector2D<f64, euclid::UnknownUnit>;

const MUSICIAN_R: f64 = 5.;

#[derive(Clone, Debug)]
pub struct Board {
    pub problem_id: u32,
    pub solver: String,
    // NB: stage is modified
    pub prob: Problem,

    // m -> position
    // musicians + pillars
    ps: Vec<Option<(P, f64)>>,
    // m -> important audience ids sorted by args
    aids: Vec<Vec<(F64, usize)>>,

    // m -> a -> j s.t. aids[j].1 == a
    aids_rev: Vec<Vec<Option<usize>>>,

    // m -> j -> block count of aids[j].1
    blocks: Vec<Vec<u8>>,
    // m -> closeness factor
    qs: Vec<f64>,
    // m -> I
    impacts: Vec<f64>,

    // m -> j -> impact between m and aids[j].1.
    individual_impacts: Vec<Vec<i64>>,

    // m -> a -> visibility [0., 1.] (0. == completely invisible)
    visibility: Vec<Vec<f64>>,
    use_visibility: bool,

    // ins -> an available musician
    available_musician: Vec<Option<usize>>,

    volumes: Vec<f64>,

    options: BoardOptions,
}

impl Board {
    pub fn new<T: AsRef<str>>(
        problem_id: u32,
        prob: Problem,
        solver: T,
        use_visibility: bool,
    ) -> Self {
        Self::new_with_options(problem_id, prob, solver, use_visibility, Default::default())
    }

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
        let aids = vec![vec![]; n];
        let aids_rev = vec![vec![None; m]; n];

        let blocks = vec![vec![0; m]; n];
        let qs = vec![1.; n];
        let impacts = vec![0.; n];
        let individual_impacts = vec![vec![0; m]; n];
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
        for (j, b) in self.blocks[m].iter().enumerate() {
            if *b > 0 {
                continue;
            }
            res += self.individual_impacts[m][j];
        }
        res as f64
    }

    pub fn contribution2(&self, m: usize) -> f64 {
        // (self.volumes[m] * self.qs[m] * self.impacts[m]).ceil()

        (self.qs[m] * self.impacts[m]).ceil()
    }

    pub fn contribution_if_instrument(&self, m: usize, ins: usize) -> f64 {
        let mut res = 0.;
        for (j, b) in self.blocks[m].iter().enumerate() {
            if *b > 0 {
                continue;
            }
            res += self.impact_if_kind(m, j, ins);
        }
        res
    }

    pub fn contribution_for(&self, m: usize, a: usize) -> f64 {
        let Some(j) = self.aids_rev[m][a] else {return 0.0};

        if self.blocks[m][j] > 0 {
            return 0.;
        }
        self.individual_impacts[m][j] as f64
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
        let Some(j) = self.aids_rev[m][a] else { return false };
        return self.blocks[m][j] == 0;
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
                    bail!("too close to another musician");
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

        assert!(self.aids[m].is_empty());

        // Update qs
        self.update_qs(m, true);

        // Update aids
        let mut max_dist2 = None;
        if self.options.important_attendees_ratio < 1.0 {
            let mut dists = self
                .prob
                .attendees
                .iter()
                .map(|a| (a.position - p).to_vector().square_length())
                .collect::<Vec<_>>();
            dists.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            let max_dist2_idx = ((dists.len() as f64 * self.options.important_attendees_ratio)
                .ceil() as usize)
                .min(dists.len() - 1);
            max_dist2 = Some(dists[max_dist2_idx]);
        }

        for (i, a) in self.prob.attendees.iter().enumerate() {
            let r: f64 = (a.position - p).to_vector().angle_from_x_axis().radians;

            let is_important = if let Some(max_dist2) = max_dist2 {
                let dist = (a.position - p).to_vector().square_length();
                dist <= max_dist2
            } else {
                true
            };

            if is_important {
                self.aids[m].push((r.into(), i));
            }
        }
        self.aids[m].sort_unstable();

        self.aids_rev[m].fill(None);
        for j in 0..self.aids[m].len() {
            self.aids_rev[m][self.aids[m][j].1] = j.into();
        }

        self.impacts[m] = 0.0;
        for j in 0..self.aids[m].len() {
            self.individual_impacts[m][j] = self.impact(m, j) as i64;
            self.impacts[m] += self.individual_impacts[m][j] as f64;
        }

        // Update blocks
        self.update_blocks(m, p, true);

        self.update_available_musician(m);
    }

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

        // Update aids
        self.aids[m].clear();

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

        let mut rs = vec![];

        for (i, q) in ps.iter().enumerate() {
            if i == m {
                continue;
            }

            let Some((q, r)) = q else { continue; };

            for rev in [false, true] {
                let (blocking, blocked, blocked_i, blocking_i, _, r, blocked_aids) = if rev {
                    (*q, p, m, i, MUSICIAN_R, *r, &all_aids[m])
                } else {
                    if i >= prob.musicians.len() {
                        // blocked is pillar, we don't need to calculate impact
                        continue;
                    }
                    (p, *q, i, m, *r, MUSICIAN_R, &all_aids[i])
                };

                // Update for blocked musician.

                let (t1, t2) = tangent_to_circle(blocked, blocking, r);

                let eps = 1e-12;
                let r1: f64 = (t1 - blocked).angle_from_x_axis().radians + eps;
                let r2: f64 = (t2 - blocked).angle_from_x_axis().radians - eps;

                let (r1, r2) = (F64::new(r1), F64::new(r2));

                let j1 = blocked_aids.partition_point(|r| r.0 < r1);

                rs.clear();
                if r1 < r2 {
                    rs.push((r1, r2));
                } else {
                    rs.push((r1, F64::new(2. * PI + eps)));
                    rs.push((F64::new(-2. * PI - eps), r2));
                }

                let distance_to_blocker_sq = (blocked - blocking).square_length();

                for (ri, (r1, r2)) in rs.iter().enumerate() {
                    let j1 = if ri == 0 { j1 } else { 0 };

                    for j in j1..blocked_aids.len() {
                        let (r, a) = &blocked_aids[j];

                        if r > r2 {
                            break;
                        }

                        let distance_to_attendee_sq = (self.prob.attendees[*a].position - blocked)
                            .to_vector()
                            .square_length();

                        let vis = if self.use_visibility {
                            let vis = 1.
                                - (r2.get() - r.get()).min(r.get() - r1.get())
                                    / ((r2.get() - r1.get()) / 2.)
                                + 1e-6;
                            let vis = Self::vis_f(vis);
                            vis
                        } else {
                            0.0
                        };

                        if distance_to_attendee_sq <= distance_to_blocker_sq {
                            continue;
                        }

                        if inc {
                            let b = &mut blocks[blocked_i][j];
                            *b += 1;
                            let impact = individual_impacts[blocked_i][j] as f64;

                            if self.use_visibility {
                                let prev_vis = self.visibility[blocked_i][*a];
                                self.visibility[blocked_i][*a] *= vis;
                                impacts[blocked_i] +=
                                    (self.visibility[blocked_i][*a] - prev_vis) * impact;
                            } else {
                                if *b == 1 {
                                    impacts[blocked_i] -= impact;
                                }
                            }
                        } else {
                            let b = &mut blocks[blocked_i][j];
                            *b -= 1;

                            let impact = individual_impacts[blocked_i][j] as f64;
                            if self.use_visibility {
                                let prev_vis = self.visibility[blocked_i][*a];
                                self.visibility[blocked_i][*a] /= vis;

                                impacts[blocked_i] +=
                                    (self.visibility[blocked_i][*a] - prev_vis) * impact;
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
                    for (r, a) in blocked_aids.iter() {
                        let seg = LineSegment {
                            from: self.prob.attendees[*a].position,
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
                                *a, self.prob.attendees[*a].position,
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

        if self.available_musician[ins].unwrap() != m {
            return;
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
        let d2 = (self.prob.attendees[self.aids[m][j].1].position - self.ps[m].unwrap().0)
            .to_vector()
            .square_length();

        let impact = 1_000_000.0 * self.prob.attendees[self.aids[m][j].1].tastes[k] / d2;
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
        for important_attendees_ratio in [1.0, 0.99] {
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

            let expected_score = evaluate(&problem, &solution);

            assert_eq!(board.score(), expected_score);

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
