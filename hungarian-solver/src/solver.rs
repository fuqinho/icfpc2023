use std::collections::BTreeSet;

use anyhow::Context;
use common::{board::Board, Problem};
use lyon_geom::Point;
use pathfinding::prelude::{kuhn_munkres, Matrix, Weights};

pub struct Solver {
    orig_problem: Problem,
    board: Board,
}

const D: usize = 10;

impl Solver {
    pub fn new(problem_id: u32, problem: Problem) -> Self {
        let board = Board::new(problem_id, problem.clone());

        Self {
            orig_problem: problem,
            board,
        }
    }

    pub fn solve(&mut self) -> (f64, Board) {
        let mut outer = vec![];

        let bb = self.board.prob.stage;

        for x in ((bb.min.x.ceil() as usize)..(bb.max.x.floor() as usize)).step_by(D) {
            outer.push(Point::new(x as f64, bb.min.y));
            outer.push(Point::new(x as f64, bb.max.y));
        }

        for y in ((bb.min.y).ceil() as usize + D..bb.max.y.floor() as usize - D).step_by(D) {
            outer.push(Point::new(bb.min.x, y as f64));
            outer.push(Point::new(bb.max.x, y as f64));
        }

        // Compute scores for outer points
        let mut scores = vec![vec![]; outer.len()];

        {
            let mut prob2 = self.orig_problem.clone();
            let num_instruments = prob2.attendees[0].tastes.len();
            prob2.musicians = vec![];
            for i in 0..num_instruments {
                prob2.musicians.push(i);
            }
            for _ in 0..outer.len() {
                prob2.musicians.push(0);
            }
            let mut board2 = Board::new(0, prob2);

            for (i, o) in outer.iter().enumerate() {
                board2.try_place(num_instruments + i, o.clone()).unwrap();
            }
            for (i, o) in outer.iter().enumerate() {
                board2.unplace(num_instruments + i);

                for j in 0..num_instruments {
                    board2.try_place(j, o.clone()).unwrap();

                    scores[i].push(board2.contribution(j));

                    board2.unplace(j);
                }

                board2.try_place(num_instruments + i, *o).unwrap();
            }
        }

        let mut weights = vec![vec![]; outer.len()];
        for i in 0..outer.len() {
            for j in self.board.prob.musicians.iter() {
                weights[i].push(scores[i][*j] as i64);
            }
        }

        eprintln!(
            "Computing hungarian on {} x {} matrix",
            weights.len(),
            weights[0].len()
        );

        let weights = Matrix::from_rows(weights).unwrap();

        let (score, assignments) = kuhn_munkres(&weights);

        let mut to_place: BTreeSet<usize> = (0..self.board.prob.musicians.len()).collect();

        for (i, j) in assignments.iter().enumerate() {
            self.board
                .try_place(*j, outer[i])
                .context("failed to place")
                .unwrap();

            to_place.remove(j);
        }

        for x in ((bb.min.x.ceil() as usize)..(bb.max.x.floor() as usize - D)).step_by(D) {
            if to_place.is_empty() {
                break;
            }

            for y in ((bb.min.y).ceil() as usize..bb.max.y.floor() as usize - D).step_by(D) {
                if to_place.is_empty() {
                    break;
                }

                if x == bb.min.x.ceil() as usize || y == bb.min.y.ceil() as usize {
                    continue;
                }

                let i = to_place.pop_first().unwrap();
                self.board
                    .try_place(i, Point::new(x as f64, y as f64))
                    .unwrap();
            }
        }

        assert!(to_place.is_empty());
        // assert_eq!(self.board.score(), score as f64);

        return (self.board.score() as f64, self.board.clone());
    }
}
