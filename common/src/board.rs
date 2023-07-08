use anyhow::bail;
use euclid::{Angle, Box2D, Vector2D};
use lyon_geom::Point;

use crate::{geom::tangent_to_circle, Placement, Problem, Solution};

type P = Vector2D<f64, euclid::UnknownUnit>;

const R: f64 = 5.;

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
    ps: Vec<Option<P>>,
    // m -> audience ids sorted by args
    aids: Vec<Vec<(F64, usize)>>,
    // m -> a -> block count
    blocks: Vec<Vec<usize>>,

    score: f64,
}

impl Board {
    pub fn new(problem_id: u32, mut prob: Problem) -> Self {
        let n = prob.musicians.len();
        let m = prob.attendees.len();

        let ps = vec![None; n];
        let aids = vec![vec![]; n];
        let blocks = vec![vec![0; m]; n];

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
            score: 0.,
        }
    }

    pub fn score(&self) -> f64 {
        self.score
    }

    pub fn musicians(&self) -> &Vec<Option<P>> {
        &self.ps
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

    pub fn try_place(&mut self, i: usize, position: Point<f64>) -> anyhow::Result<()> {
        let mut bb = self.prob.stage;
        bb.max += P::new(1e-9, 1e-9);
        if !bb.contains(position) {
            bail!("not on stage");
        }
        for p in self.ps.iter() {
            if let Some(p) = p {
                if (*p - position.to_vector()).length() < 10. {
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
        // Update ps
        self.ps[m] = Some(p);

        assert!(self.aids[m].is_empty());

        // Update aids and score
        for (i, a) in self.prob.attendees.iter().enumerate() {
            let r: F64 = (a.position - p)
                .to_vector()
                .angle_from_x_axis()
                .radians
                .into();
            self.aids[m].push((r, i));

            self.score += self.impact(m, i)
        }
        self.aids[m].sort_unstable();

        // Update blocks
        self.update_blocks(m, p, true);
    }

    pub fn can_place(&self, i: usize, position: Point<f64>) -> bool {
        for (ix, p) in self.ps.iter().enumerate() {
            if ix == i {
                continue;
            }
            if let Some(p) = p {
                if (*p - position.to_vector()).length() < 10. {
                    return false;
                }
            }
        }
        true
    }

    pub fn unplace(&mut self, m: usize) {
        let p = self.ps[m].unwrap();

        // Update blocks
        self.update_blocks(m, p, false);

        // Update aids and score
        self.aids[m].clear();
        for (i, _) in self.prob.attendees.iter().enumerate() {
            self.score -= self.impact(m, i)
        }

        // Update ps
        self.ps[m] = None;
    }

    fn update_blocks(&mut self, m: usize, p: P, inc: bool) {
        for (i, q) in self.ps.clone().into_iter().enumerate() {
            if i == m {
                continue;
            }

            if let Some(q) = q {
                for rev in [false, true] {
                    let (blocking, blocked, _, i) = if rev { (q, p, i, m) } else { (p, q, m, i) };

                    // Update for blocked musician.

                    let (t1, t2) = tangent_to_circle(blocked, blocking, R);

                    let r1: F64 = (t1 - blocked).angle_from_x_axis().radians.into();
                    let r2: F64 = (t2 - blocked).angle_from_x_axis().radians.into();

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
            self.score -= self.impact(i, a);
        }
    }

    fn dec_blocks(&mut self, i: usize, a: usize) {
        let b = &mut self.blocks[i][a];
        *b -= 1;
        if *b == 0 {
            self.score += self.impact(i, a);
        }
    }

    fn impact(&self, m: usize, a: usize) -> f64 {
        let d2 = (self.prob.attendees[a].position - self.ps[m].unwrap())
            .to_vector()
            .square_length();
        let impact = 1_000_000.0 * self.prob.attendees[a].tastes[self.prob.musicians[m]] / d2;
        impact.ceil()
    }
}

impl TryInto<Solution> for Board {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Solution, Self::Error> {
        let mut placements = vec![];
        for p in self.ps {
            if let Some(p) = p {
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
    use lyon_geom::Point;
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::{board::Board, evaluate, Problem, Solution};

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
}
