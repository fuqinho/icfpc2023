use anyhow::bail;
use euclid::{Box2D, Vector2D};
use lyon_geom::Point;

use crate::{geom::tangent_to_circle, Placement, Problem, Solution};

type P = Vector2D<f64, euclid::UnknownUnit>;

const MUSICIAN_R: f64 = 5.;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct F64(pub f64);

impl Eq for F64 {}

impl Ord for F64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl From<f64> for F64 {
    fn from(f: f64) -> Self {
        Self(f)
    }
}

#[derive(Clone, Debug)]
pub struct Board {
    problem_id: u32,
    // NB: stage is modified
    pub prob: Problem,

    // m -> position
    // musicians + pillars
    ps: Vec<Option<(P, f64)>>,
    // m -> audience ids sorted by args
    aids: Vec<Vec<(F64, usize)>>,
    // m -> a -> block count
    blocks: Vec<Vec<usize>>,
    // m -> closeness factor
    qs: Vec<f64>,
    // m -> I
    impacts: Vec<f64>,
}

impl Board {
    pub fn new(problem_id: u32, mut prob: Problem) -> Self {
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

        prob.stage = Box2D::new(
            prob.stage.min + P::new(10., 10.),
            prob.stage.max - P::new(10., 10.),
        );

        Self {
            problem_id,
            prob,
            ps,
            aids,
            blocks,
            qs,
            impacts,
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
            let r: F64 = (a.position - p)
                .to_vector()
                .angle_from_x_axis()
                .radians
                .into();
            self.aids[m].push((r, i));
        }
        self.aids[m].sort_unstable();

        // Update blocks
        self.update_blocks(m, p, true);
    }

    pub fn can_place(&self, i: usize, position: Point<f64>) -> bool {
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
        for (i, q) in self.ps.clone().into_iter().enumerate() {
            if i == m {
                continue;
            }

            if let Some((q, r)) = q {
                for rev in [false, true] {
                    let (blocking, blocked, _, i, _, r) = if rev {
                        (q, p, i, m, r, MUSICIAN_R)
                    } else {
                        (p, q, m, i, MUSICIAN_R, r)
                    };
                    if i >= self.prob.musicians.len() {
                        // blocked is pillar, we don't need to calculate impact
                        continue;
                    }

                    // Update for blocked musician.

                    let (t1, t2) = tangent_to_circle(blocked, blocking, r);

                    let mut r1: F64 = (t1 - blocked).angle_from_x_axis().radians.into();
                    let mut r2: F64 = (t2 - blocked).angle_from_x_axis().radians.into();

                    let eps = 1e-12;
                    r1.0 += eps;
                    r2.0 -= eps;

                    let mut rs = vec![];

                    if r1 < r2 {
                        rs.push((r1, r2));
                    } else {
                        rs.push((r1, (std::f64::consts::PI).into()));
                        rs.push(((-std::f64::consts::PI).into(), r2));
                    }

                    for (r1, r2) in rs {
                        let j1 = self.aids[i].binary_search(&(r1, 0)).unwrap_or_else(|j| j);
                        let j2 = self.aids[i].binary_search(&(r2, 0)).unwrap_or_else(|j| j);

                        for j in j1..j2 {
                            if inc {
                                self.inc_blocks(i, self.aids[i][j].1);
                            } else {
                                self.dec_blocks(i, self.aids[i][j].1);
                            }
                        }
                    }
                }
            }
        }
    }

    fn inc_blocks(&mut self, i: usize, a: usize) {
        let b = &mut self.blocks[i][a];
        *b += 1;
        if *b == 1 {
            self.impacts[i] -= self.impact(i, a);
        }
    }

    fn dec_blocks(&mut self, i: usize, a: usize) {
        let b = &mut self.blocks[i][a];
        *b -= 1;
        if *b == 0 {
            self.impacts[i] += self.impact(i, a);
        }
    }

    fn impact(&self, m: usize, a: usize) -> f64 {
        self.impact_if_kind(m, a, self.prob.musicians[m])
    }

    fn impact_if_kind(&self, m: usize, a: usize, k: usize) -> f64 {
        let d2 = (self.prob.attendees[a].position - self.ps[m].unwrap().0)
            .to_vector()
            .square_length();
        let impact = 1_000_000.0 * self.prob.attendees[a].tastes[k] / d2;
        impact.ceil()
    }
}

impl TryInto<Solution> for Board {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Solution, Self::Error> {
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
            placements,
        })
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

        let mut board = Board::new(problem_id, problem.clone());

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

            let mut board = Board::new(0, problem.clone());

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
