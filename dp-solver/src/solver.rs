use common::{board::Board, geom::tangent_circle2, Attendee, Problem};
use lyon_geom::{LineSegment, Vector};

const SOLVER_NAME: &str = "dp-solver";

const D: usize = 2;

pub fn solve(problem_id: u32, problem: Problem) -> (f64, Board) {
    if problem.stage.min.x > 0. || problem.stage.min.y > 0. {
        panic!("stage.min should be (0, 0)");
    }

    let mul_problem = problem.multiplied(D as f64);

    let mul_problem_flipped = mul_problem.flipped();

    let p1 = Dp::new(mul_problem).solve();
    let mut p2 = Dp::new(mul_problem_flipped).solve();

    p2.iter_mut().for_each(|p| *p = P::new(p.y, p.x));

    let mut ps = p1;
    ps.append(&mut p2);

    ps.iter_mut().for_each(|p| {
        p.x /= D as f64;
        p.y /= D as f64;
    });

    let mut conflicts = vec![];

    for i in 0..ps.len() {
        for j in 0..ps.len() {
            if i == j {
                continue;
            }
            if (ps[i] - ps[j]).square_length() < 100. {
                conflicts.push(i);
                break;
            }
        }
    }

    let mut masks = vec![];
    for mask in 0..1 << conflicts.len() {
        let mut ok = true;
        for i in 0..conflicts.len() {
            for j in 0..i {
                if mask & 1 << i == 0 && mask & 1 << j == 0 {
                    if (ps[conflicts[i]] - ps[conflicts[j]]).square_length() < 100. {
                        ok = false;
                    }
                }
            }
        }
        if ok {
            masks.push(mask);
        }
    }

    let mut minimal_masks = vec![];
    for m1 in masks.iter() {
        let mut is_min = true;
        for m2 in masks.iter() {
            if m1 != m2 && m1 & m2 == *m2 {
                is_min = false;
                break;
            }
        }
        if is_min {
            minimal_masks.push(m1);
        }
    }

    let mut best: Option<(f64, Board)> = None;

    for mask in minimal_masks {
        let mut outer = vec![];

        for (i, p) in ps.iter().enumerate() {
            let mut masked = false;
            for j in 0..conflicts.len() {
                if mask & 1 << j != 0 && i == conflicts[j] {
                    masked = true;
                }
            }
            if !masked {
                outer.push(*p);
            }
        }

        let mut hs = hungarian_solver::solver::Solver::new(
            problem_id,
            problem.clone(),
            hungarian_solver::solver::Algorithm::Normal,
        );
        let (score, board) =
            hs.solve_with_positions(&outer.into_iter().map(|p| p.to_point()).collect());

        if best.is_none() || score > best.as_ref().unwrap().0 {
            best = Some((score, board));
        }
    }

    let (score, mut board) = best.unwrap();

    board.solver = SOLVER_NAME.to_string();

    (score, board)
}

pub struct Dp {
    // height of the stage (max y)
    h: usize,
    // width of the stage. x to put on the circle.
    w: f64,
    // num instruments (i)
    k: usize,

    d: usize,
    r: f64,

    // num attendees (a)
    attendees: Vec<Attendee>,

    // y -> i -> d -> y's blocked impact by y + 10 + d.
    blk_pos: Vec<Vec<Vec<i64>>>,
    // y -> i -> d -> y's blocked impact by y - 10 - d.
    blk_neg: Vec<Vec<Vec<i64>>>,

    // y -> d -> best score by back musician between y and y + 10 + d.
    add: Vec<Vec<i64>>,

    // y -> i -> total impacts on 180 degrees
    all: Vec<Vec<i64>>,

    // y -> i -> best score by putting instrument i on y.
    dp: Vec<Vec<i64>>,
}

type P = Vector<f64>;

impl Dp {
    pub fn new(problem: Problem) -> Self {
        let h = problem.stage.max.y as usize - 10 * D;
        let w = problem.stage.max.x - 10. * D as f64;
        let k = problem.attendees[0].tastes.len();
        let d = 10 * D;

        let attendees = problem
            .attendees
            .clone()
            .into_iter()
            .filter(|a| a.position.x >= w)
            .collect();

        let blk_pos = vec![vec![vec![0; d]; k]; h + 1];
        let blk_neg = vec![vec![vec![0; d]; k]; h + 1];
        let add = vec![vec![0; d]; h + 1];
        let all = vec![vec![0; k]; h + 1];

        let dp = vec![vec![i64::MIN / 2; k]; h + 1];

        Self {
            h,
            w,
            k,
            d,
            r: d as f64 / 2.,
            attendees,
            blk_pos,
            blk_neg,
            add,
            all,
            dp,
        }
    }

    pub fn init(&mut self) {
        // all
        for y in 0..=self.h {
            let p = self.point(y);
            for i in 0..self.k {
                for a in self.attendees.iter() {
                    self.all[y][i] += impact(a, i, p);
                }
            }
        }
        // blk_pos, blk_neg
        for y in 0..=self.h {
            let p = self.point(y);

            for i in 0..self.k {
                for d in 0..self.d {
                    if y + self.d + d > self.h {
                        break;
                    }

                    let q = self.point(y + self.d + d);

                    for a in self.attendees.iter() {
                        if y > 0 && !is_visible(a, p, q) {
                            self.blk_pos[y][i][d] += impact(a, i, p);
                        }
                        if !is_visible(a, q, p) {
                            self.blk_neg[y + self.d + d][i][d] += impact(a, i, q);
                        }
                    }
                }
            }
        }
        // add
        for y in 1..=self.h {
            for d in 0..self.d {
                let mut best = 0;

                for ins in 0..self.k {
                    let p = self.point(y);
                    let q = self.point(y + self.d + d);

                    let r = self.r + 1e-6;

                    let tc = tangent_circle2(p, q, self.r, r).unwrap();

                    if tc.y < self.d as f64 {
                        continue;
                    }

                    let min_arg = (p - tc).angle_from_x_axis().radians;
                    let max_arg = (q - tc).angle_from_x_axis().radians;

                    let mut v = 0;
                    for a in self.attendees.iter() {
                        let arg = (a.position - tc).to_vector().angle_from_x_axis().radians;

                        if arg < min_arg || arg > max_arg {
                            continue;
                        }

                        if is_visible(a, tc, p) && is_visible(a, tc, q) {
                            v += impact(a, ins, tc);
                        }
                    }
                    best = best.max(v);
                }

                self.add[y][d] = best;
            }
        }
    }

    fn point(&self, y: usize) -> P {
        P::new(self.w, y as f64)
    }

    // Returns the points to put the musicians.
    pub fn solve(&mut self) -> Vec<P> {
        println!("Initializing...");
        self.init();

        println!("Solving dp...");

        self.dp[0][0] = 0;

        for y in 0..=self.h {
            for i in 0..self.k {
                for d in 0..self.d {
                    if y < d + self.d {
                        continue;
                    }
                    let y0 = y - d - self.d;
                    for j in 0..self.k {
                        let v = self.dp[y0][j] - self.blk_pos[y0][j][d] + self.all[y][i]
                            - self.blk_neg[y][i][d]
                            + self.add[y0][d];

                        self.dp[y][i] = self.dp[y][i].max(v);
                    }
                }
            }
        }

        // Reconstruct
        let mut y_i = (0, 0);
        for y in 0..=self.h {
            for i in 0..self.k {
                if self.dp[y][i] > self.dp[y_i.0][y_i.1] {
                    y_i = (y, i);
                }
            }
        }

        let mut outer = vec![];
        let mut inner = vec![];

        while y_i.0 > 0 {
            let (y, i) = y_i;

            outer.push(self.point(y));

            let mut found = false;

            'outer: for d in 0..self.d {
                if y < d + self.d {
                    continue;
                }
                let y0 = y - d - self.d;
                for j in 0..self.k {
                    let v = self.dp[y0][j] - self.blk_pos[y0][j][d] + self.all[y][i]
                        - self.blk_neg[y][i][d]
                        + self.add[y0][d];

                    if self.dp[y][i] == v {
                        let r = self.r + 1e-6;

                        let tc = tangent_circle2(self.point(y0), self.point(y), self.r, r).unwrap();

                        if tc.y >= self.d as f64 {
                            inner.push(tc);
                        }

                        found = true;

                        y_i = (y0, j);
                        break 'outer;
                    }
                }
            }

            if !found {
                panic!("prev state not found: {} {}", y, i);
            }
        }

        [outer, inner].concat()
    }
}

fn is_visible(a: &Attendee, from: P, blocker: P) -> bool {
    LineSegment {
        from: from.to_point(),
        to: blocker.to_point(),
    }
    .square_distance_to_point(a.position)
        >= 25. * D as f64 * D as f64
}

fn impact(a: &Attendee, ins: usize, p: P) -> i64 {
    let ap = a.position.to_vector();
    let taste = a.tastes[ins];

    let impact = 1e6 * D as f64 * D as f64 * taste / (ap - p).square_length();

    impact.ceil() as i64
}
