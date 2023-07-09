use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
    str::FromStr,
};

use anyhow::{bail, Context};
use common::{board::Board, geom::tangent_circle, Problem};
use lyon_geom::{Point, Vector};
use pathfinding::prelude::{kuhn_munkres, Matrix};

pub struct Solver {
    orig_problem: Problem,
    board: Board,
}

type P = Point<f64>;

const D: usize = 10;

impl Solver {
    pub fn new(problem_id: u32, problem: Problem) -> Self {
        let board = Board::new(problem_id, problem.clone());

        Self {
            orig_problem: problem,
            board,
        }
    }

    pub fn solve(&mut self, algo: Algorithm) -> (f64, Board) {}
}
