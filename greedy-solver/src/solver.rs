// use common::{board::Board, Problem};
// use lyon_geom::Point;

// pub struct Solver {
//     num_musicians: usize,   // num musicians (m)
//     num_attendees: usize,   // num attendees (a)
//     num_instruments: usize, // num instruments (i)

//     // m -> maximal points sorted by score. (smaller first)
//     maximal_points: Vec<Vec<(f64, P)>>,

//     board: Board,
// }

// type P = Point<f64>;

// const D: usize = 10;

// impl Solver {
//     pub fn new(problem_id: u32, problem: Problem) -> Self {
//         let board = Board::new(problem_id, problem, "greedy-solver");

//         let num_musicians = board.prob.musicians.len();
//         let num_attendees = board.prob.attendees.len();
//         let num_instruments = board.prob.attendees[0].tastes.len();

//         let maximal_points = vec![];

//         Self {
//             num_musicians,
//             num_attendees,
//             num_instruments,
//             maximal_points,
//             board,
//         }
//     }

//     fn prob(&self) -> &Problem {
//         &self.board.prob
//     }

//     fn compute_maximal_points(&mut self) {
//         let mut maximal_points = vec![vec![]; self.num_musicians];

//         for i in 0..self.num_musicians {
//             let musician = self.prob().musicians[i];

//             for a in 0..self.num_attendees {
//                 let attendee = self.prob().attendees[a];

//                 let mut score = 0.;

//                 for j in 0..self.num_instruments {
//                     let taste = attendee.tastes[j];
//                     let musician_has = musician.has[j];

//                     if taste && musician_has {
//                         score += 1.;
//                     }
//                 }

//                 maximal_points[i].push((score, attendee.pos));
//             }

//             maximal_points[i].sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
//         }

//         self.maximal_points = maximal_points;
//     }

//     pub fn solve(&mut self) -> (f64, Board) {
//         let mut scores = vec![vec![]];

//         // for i in self.prob.

//         todo!()
//     }
// }
