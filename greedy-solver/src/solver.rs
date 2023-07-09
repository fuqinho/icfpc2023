use common::{board::Board, Problem};
use lyon_geom::Point;

pub struct Solver {
    num_musicians: usize,   // num musicians (m)
    num_attendees: usize,   // num attendees (a)
    num_instruments: usize, // num instruments (i)

    // i -> maximal points sorted by score. (smaller first)
    maximal_points: Vec<Vec<(f64, P)>>,

    board: Board,
}

type P = Point<f64>;

const D: usize = 10;

impl Solver {
    pub fn new(problem_id: u32, problem: Problem) -> Self {
        let board = Board::new(problem_id, problem, "greedy-solver");

        let num_musicians = board.prob.musicians.len();
        let num_attendees = board.prob.attendees.len();
        let num_instruments = board.prob.attendees[0].tastes.len();

        let maximal_points = vec![];

        Self {
            num_musicians,
            num_attendees,
            num_instruments,
            maximal_points,
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

    fn is_on_stage(&self, p: Point<f64>) -> bool {
        p.x >= self.left() && p.x <= self.right() && p.y >= self.bottom() && p.y <= self.top()
    }

    fn init_maximal_points(&mut self) {
        let mut maximal_points = vec![vec![]; self.num_instruments];

        for i in 0..self.num_instruments {
            let m = self.board.available_musician_with_instrument(i).unwrap();

            let delta = 0.125;

            for (dx, dy, x, y) in [
                (delta, 0., 0., self.bottom()),
                (0., delta, self.left(), 0.),
                (delta, 0., 0., self.top()),
                (0., delta, self.right(), 0.),
            ] {
                let mut graph = vec![];

                let mut p = Point::new(x, y);

                while self.is_on_stage(p) {
                    let score = self.board.score_increase_if_put_musician_on(m, p).unwrap();

                    graph.push((score, p));

                    p.x += dx;
                    p.y += dy;
                }

                for i in 0..graph.len() {
                    if i > 0 && graph[i].0 < graph[i - 1].0 {
                        continue;
                    }
                    if i + 1 < graph.len() && graph[i].0 <= graph[i + 1].0 {
                        continue;
                    }
                    maximal_points[i].push(graph[i]);
                }
            }

            maximal_points[i].sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }

        self.maximal_points = maximal_points;
    }

    // pub fn musician_for_instrument(&mut self, ins: usize) -> Option<usize> {
    //     if self.unavailable[ins] {
    //         return None;
    //     }
    //     let res = self.board.available_musician_with_instrument(ins);
    //     if res.is_none() {
    //         self.unavailable[ins] = true;
    //     }
    //     res
    // }

    pub fn solve(&mut self) -> (f64, Board) {
        self.init_maximal_points();

        // Put all the points greedily.

        todo!()
    }
}
