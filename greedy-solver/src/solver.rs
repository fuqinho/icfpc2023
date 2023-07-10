use anyhow::Context;
use common::{
    board::Board,
    geom::{
        circles_tangenting_line_and_circle, circles_tangenting_lines, new_line, tangent_circle,
    },
    Problem,
};
use lyon_geom::{LineSegment, Point, Vector};

pub struct Solver {
    num_musicians: usize,   // num musicians (m)
    num_attendees: usize,   // num attendees (a)
    num_instruments: usize, // num instruments (i)

    // i -> maximal points sorted by score. (smaller first)
    maximal_points: Vec<Vec<(f64, P)>>,

    // reverse sorted by impact
    important_segs: Vec<(f64, P, P)>,

    board: Board,
}

type P = Vector<f64>;

const D: usize = 10;

const KEEP_IMPORTANT_SEGS: usize = 100;
const USE_IMPORTANT_SEGS: usize = 50;

const EPS: f64 = 1e-9;

const R: f64 = 5.0;

const INITIAL_SEARCH_STEP: f64 = 2.0;

impl Solver {
    pub fn new(problem_id: u32, problem: Problem) -> Self {
        let board = Board::new(problem_id, problem, "greedy-solver", false);

        let num_musicians = board.prob.musicians.len();
        let num_attendees = board.prob.attendees.len();
        let num_instruments = board.prob.attendees[0].tastes.len();

        let maximal_points = vec![];

        Self {
            num_musicians,
            num_attendees,
            num_instruments,
            maximal_points,
            important_segs: vec![],
            board,
        }
    }

    fn prob(&self) -> &Problem {
        &self.board.prob
    }

    fn top(&self) -> f64 {
        self.board.prob.stage.max.y
    }

    fn bottom(&self) -> f64 {
        self.board.prob.stage.min.y
    }

    fn right(&self) -> f64 {
        self.board.prob.stage.max.x
    }

    fn left(&self) -> f64 {
        self.board.prob.stage.min.x
    }

    fn is_on_stage(&self, p: P) -> bool {
        p.x >= self.left() && p.x <= self.right() && p.y >= self.bottom() && p.y <= self.top()
    }

    fn is_narrow(&self) -> bool {
        self.top() == self.bottom() || self.left() == self.right()
    }

    fn init_maximal_points(&mut self) {
        let mut maximal_points = vec![vec![]; self.num_instruments];

        for ins in 0..self.num_instruments {
            eprintln!("initializing max point: {} / {}", ins, self.num_instruments);

            let m = self.board.available_musician_with_instrument(ins).unwrap();

            let delta = INITIAL_SEARCH_STEP;

            for (dx, dy, x, y) in [
                (delta, 0., self.left(), self.bottom()),
                (delta, 0., self.left(), self.top()),
                (0., delta, self.left(), self.bottom()),
                (0., delta, self.right(), self.bottom()),
            ] {
                let mut graph = vec![];

                let mut p = P::new(x, y);

                while self.is_on_stage(p) {
                    let score = self
                        .board
                        .score_increase_if_put_musician_on(m, p.to_point())
                        .unwrap();

                    graph.push((score, p));

                    p.x += dx;
                    p.y += dy;
                }

                assert!(!graph.is_empty());

                for i in 0..graph.len() {
                    if i > 0 && graph[i].0 < graph[i - 1].0 {
                        continue;
                    }
                    if i + 1 < graph.len() && graph[i].0 <= graph[i + 1].0 {
                        continue;
                    }
                    maximal_points[ins].push(graph[i]);
                }
            }

            eprintln!(
                "number of maximal points for instrument {}: {}",
                ins,
                maximal_points[ins].len()
            );

            maximal_points[ins].sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }

        self.maximal_points = maximal_points;
    }

    fn init_important_segs(&mut self) {
        let left = self.left() - R;
        let right = self.right() + R;
        let bottom = self.bottom() - R;
        let top = self.top() + R;

        let p00 = P::new(left, bottom);
        let p01 = P::new(left, top);
        let p10 = P::new(right, bottom);
        let p11 = P::new(right, top);

        self.important_segs.push((f64::INFINITY, p00, p01));
        self.important_segs.push((f64::INFINITY, p00, p10));
        self.important_segs.push((f64::INFINITY, p01, p11));
        self.important_segs.push((f64::INFINITY, p10, p11));
    }

    pub fn solve(&mut self) -> (f64, Board) {
        self.init_maximal_points();
        self.init_important_segs();

        let mut score_inc = vec![];
        let mut score = 0.0;

        // Put all the points greedily.
        for iter in 0..self.num_musicians {
            eprintln!("iter {} / {}", iter, self.num_musicians);

            let mut best = (f64::NEG_INFINITY, 0, 0, P::new(0., 0.));

            for ins in 0..self.num_instruments {
                if let Some(m) = self.board.available_musician_with_instrument(ins) {
                    assert_eq!(self.board.prob.musicians[m], ins);

                    let (score, p) = self.best_position(m, ins);

                    if best.0 < score {
                        best = (score, m, ins, p);
                    }
                }
            }

            let (_, m, ins, p) = best;

            self.place(m, ins, p);

            score_inc.push((best.1, self.board.score() - score));
            score = self.board.score();
        }

        let mut acc = 0.;
        for (i, (m, inc)) in score_inc.into_iter().enumerate() {
            eprintln!(
                "iter {}: musician {} got score {} = {:.2}% in total (accum {:.2}%)",
                i,
                m,
                inc,
                inc / score * 100.,
                acc / score * 100.
            );
            acc += inc;
        }

        (self.board.score(), self.board.clone())
    }

    fn best_position(&mut self, m: usize, ins: usize) -> (f64, P) {
        let cps = self.candidate_points(m, ins);

        let mut best = (f64::NEG_INFINITY, P::new(0., 0.));

        for cp in cps {
            let score = self
                .board
                .score_increase_if_put_musician_on(m, cp.to_point())
                .unwrap();

            if best.0 < score {
                best = (score, cp);
            }
        }

        best
    }

    fn candidate_points(&mut self, m: usize, ins: usize) -> Vec<P> {
        let mut res = vec![];

        // Pop unusable maximal points.
        loop {
            let lst = self.maximal_points[ins].last();

            if lst == None {
                break;
            }

            let (_, p) = lst.unwrap();

            if self.board.can_place(m, p.to_point()) {
                res.push(*p);
                break;
            }

            self.maximal_points[ins].pop();
        }

        // Tangent to two important segs
        let r = if self.is_narrow() { R } else { R + EPS };

        for i in 0..self.important_segs.len().min(USE_IMPORTANT_SEGS) {
            for j in 0..i {
                let (_, p0, p1) = self.important_segs[i];
                let (_, q0, q1) = self.important_segs[j];

                for c in circles_tangenting_lines(p0, p1, q0, q1, r) {
                    if self.can_place(m, c) {
                        res.push(c);
                    }
                }
            }
        }

        // Tangent to an important seg and a circle
        for i in 0..self.important_segs.len().min(USE_IMPORTANT_SEGS) {
            let (_, p0, p1) = self.important_segs[i];

            for mc in self.board.musicians() {
                if let Some((mc, _)) = mc {
                    for c in circles_tangenting_line_and_circle(p0, p1, *mc, R, r) {
                        if self.can_place(m, c) {
                            res.push(c);
                        }
                    }
                }
            }
        }

        // Tangent to two circles
        for mc1 in self.board.musicians() {
            if let Some((mc1, _)) = mc1 {
                for mc2 in self.board.musicians() {
                    if let Some((mc2, _)) = mc2 {
                        if mc1 == mc2 {
                            continue;
                        }

                        let c = tangent_circle(*mc1, *mc2, r);

                        if let Some(c) = c {
                            if self.can_place(m, c) {
                                res.push(c);
                            }
                        }
                    }
                }
            }
        }

        res
    }

    fn can_place(&self, m: usize, p: P) -> bool {
        self.is_on_stage(p) && self.board.can_place(m, p.to_point())
    }

    fn place(&mut self, m: usize, ins: usize, p: P) {
        self.board
            .try_place(m, p.to_point())
            .with_context(|| format!("failed to place musician {} at {:?}", m, p))
            .unwrap();

        // Update important segs
        let mut important_segs = vec![];

        for (imp, p1, p2) in self.important_segs.iter() {
            if (LineSegment {
                from: p1.to_point(),
                to: p2.to_point(),
            })
            .distance_to_point(p.to_point())
                >= R
            {
                important_segs.push((*imp, *p1, *p2));
            }
        }

        for a in 0..self.num_attendees {
            let c = self.board.contribution_for(m, a);

            important_segs.push((c, self.board.prob.attendees[a].position.to_vector(), p));
        }

        important_segs.sort_by(|(c1, _, _), (c2, _, _)| c1.partial_cmp(c2).unwrap());
        important_segs.reverse();

        important_segs.truncate(KEEP_IMPORTANT_SEGS);

        self.important_segs = important_segs;
    }
}
