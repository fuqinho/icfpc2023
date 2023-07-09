use anyhow::bail;
use euclid::{Box2D, Vector2D};
use lyon_geom::Point;

use crate::{geom::tangent_to_circle, Placement, Problem, Solution};

use anyhow::Result;

type P = Vector2D<f64, euclid::UnknownUnit>;

const MUSICIAN_R: f64 = 5.;

#[derive(Clone, Debug)]
pub struct Board {
    problem_id: u32,
    solver: String,
    // NB: stage is modified
    pub prob: Problem,

    // m -> position
    // musicians + pillars
    ps: Vec<Option<(P, f64)>>,
    // m -> audience ids sorted by args
    aids: Vec<Vec<(f64, usize)>>,
    // m -> a -> block count
    blocks: Vec<Vec<usize>>,
    // m -> closeness factor
    qs: Vec<f64>,
    // m -> I
    impacts: Vec<f64>,

    // ins -> an available musician
    available_musician: Vec<Option<usize>>,

    volumes: Vec<f64>,
}

impl Board {
    pub fn new(problem_id: u32, mut prob: Problem, solver: &str) -> Self {
        let n = prob.musicians.len();
        let m = prob.attendees.len();
        let p = prob.pillars.len();

        let mut ps = vec![None; n + p];
        for i in 0..p {
            ps[i + n] = Some((prob.pillars[i].center.to_vector(), prob.pillars[i].radius));
        }
        let aids = vec![vec![]; n];
        let blocks = vec![vec![0; m]; n];
        let qs = vec![1.; n];
        let impacts = vec![0.; n];

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
            solver: solver.to_owned(),
            prob,
            ps,
            aids,
            blocks,
            qs,
            impacts,
            available_musician,
            volumes: vec![1.; n],
        }
    }

    pub fn score(&self) -> f64 {
        let mut res = 0.;
        for m in 0..self.musicians().len() {
            res += self.qs[m] * self.impacts[m];
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
        // TODO(chir): score recalcuration.
    }

    // The musician's contribution to the score
    pub fn contribution(&self, m: usize) -> f64 {
        let mut res = 0.;
        for (a, b) in self.blocks[m].iter().enumerate() {
            if *b > 0 {
                continue;
            }
            res += self.impact(m, a);
        }
        res
    }

    pub fn contribution_if_instrument(&self, m: usize, ins: usize) -> f64 {
        let mut res = 0.;
        for (a, b) in self.blocks[m].iter().enumerate() {
            if *b > 0 {
                continue;
            }
            res += self.impact_if_kind(m, a, ins);
        }
        res
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

    pub fn try_place(&mut self, i: usize, position: Point<f64>) -> anyhow::Result<()> {
        let mut bb = self.prob.stage;
        bb.max += P::new(1e-9, 1e-9);
        if !bb.contains(position) {
            bail!("not on stage");
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
        self.impacts[m] = 0.0;
        for i in 0..self.prob.attendees.len() {
            self.impacts[m] += self.impact(m, i);
        }

        assert!(self.aids[m].is_empty());

        // Update qs
        self.update_qs(m, true);

        // Update aids
        for (i, a) in self.prob.attendees.iter().enumerate() {
            let r: f64 = (a.position - p).to_vector().angle_from_x_axis().radians;
            self.aids[m].push((r, i));
        }
        self.aids[m].sort_unstable_by(|a, b| a.0.total_cmp(&b.0));

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

    fn update_blocks(&mut self, m: usize, p: P, inc: bool) {
        let (prob, ps, aids, blocks, impacts) = (
            &self.prob,
            &self.ps,
            &self.aids,
            &mut self.blocks,
            &mut self.impacts,
        );

        for (i, q) in ps.iter().enumerate() {
            if i == m {
                continue;
            }

            let Some((q, r)) = q else { continue; };

            for rev in [false, true] {
                let (blocking, blocked, i, _, r, aids) = if rev {
                    (*q, p, m, *r, MUSICIAN_R, &aids[m])
                } else {
                    if i >= prob.musicians.len() {
                        // blocked is pillar, we don't need to calculate impact
                        continue;
                    }
                    (p, *q, i, MUSICIAN_R, *r, &aids[i])
                };

                // Update for blocked musician.

                let (t1, t2) = tangent_to_circle(blocked, blocking, r);

                let eps = 1e-12;
                let r1: f64 = (t1 - blocked).angle_from_x_axis().radians + eps;
                let r2: f64 = (t2 - blocked).angle_from_x_axis().radians - eps;

                if r1 < r2 {
                    let j1 = aids.partition_point(|r| r.0 < r1);
                    for (r, a) in &aids[j1..] {
                        if *r > r2 {
                            break;
                        }
                        if inc {
                            Self::inc_blocks(blocks, impacts, prob, ps, i, *a);
                        } else {
                            Self::dec_blocks(blocks, impacts, prob, ps, i, *a);
                        }
                    }
                } else {
                    for (r, a) in aids.iter() {
                        if *r > r2 {
                            break;
                        }
                        if inc {
                            Self::inc_blocks(blocks, impacts, prob, ps, i, *a);
                        } else {
                            Self::dec_blocks(blocks, impacts, prob, ps, i, *a);
                        }
                    }
                    for (r, a) in aids.iter().rev() {
                        if *r < r1 {
                            break;
                        }
                        if inc {
                            Self::inc_blocks(blocks, impacts, prob, ps, i, *a);
                        } else {
                            Self::dec_blocks(blocks, impacts, prob, ps, i, *a);
                        }
                    }
                };
            }
        }
    }

    #[inline]
    fn inc_blocks(
        blocks: &mut Vec<Vec<usize>>,
        impacts: &mut Vec<f64>,
        prob: &Problem,
        ps: &Vec<Option<(P, f64)>>,
        i: usize,
        a: usize,
    ) {
        let b = &mut blocks[i][a];
        *b += 1;
        if *b == 1 {
            impacts[i] -= Self::impact_internal(prob, ps, prob.musicians[i], i, a);
        }
    }

    #[inline]
    fn dec_blocks(
        blocks: &mut Vec<Vec<usize>>,
        impacts: &mut Vec<f64>,
        prob: &Problem,
        ps: &Vec<Option<(P, f64)>>,
        i: usize,
        a: usize,
    ) {
        let b = &mut blocks[i][a];
        *b -= 1;
        if *b == 0 {
            impacts[i] += Self::impact_internal(prob, ps, prob.musicians[i], i, a);
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

    fn impact_internal(
        prob: &Problem,
        ps: &Vec<Option<(P, f64)>>,
        k: usize,
        m: usize,
        a: usize,
    ) -> f64 {
        let d2 = (prob.attendees[a].position - ps[m].unwrap().0)
            .to_vector()
            .square_length();
        let impact = 1_000_000.0 * prob.attendees[a].tastes[k] / d2;
        impact.ceil()
    }

    #[inline]
    fn impact(&self, m: usize, a: usize) -> f64 {
        Self::impact_internal(&self.prob, &self.ps, self.prob.musicians[m], m, a)
    }

    #[inline]
    fn impact_if_kind(&self, m: usize, a: usize, k: usize) -> f64 {
        Self::impact_internal(&self.prob, &self.ps, k, m, a)
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
        })
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

    use crate::{board::Board, evaluate, Attendee, Problem, Solution};

    #[test]
    fn test_board() {
        let problem_id = 42u32;

        let mut rng = StdRng::seed_from_u64(42);

        let problem = Problem::read_from_file(format!("../problems/{}.json", 42)).unwrap();

        let mut board = Board::new(problem_id, problem.clone(), "test_solver");

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

            let mut board = Board::new(0, problem.clone(), "test_solver");

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
