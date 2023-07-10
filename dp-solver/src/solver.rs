use common::{board::Board, Problem};
use lyon_geom::Vector;

const SOLVER_NAME: &str = "dp-solver";

pub fn solve(problem_id: u32, problem: Problem) -> (f64, Board) {
    let mut solver = Dp::new(problem_id, problem);

    solver.solve()
}

pub struct Dp {
    orig_problem: Problem,
    board: Board,

    // height of the stage (max y)
    h: usize,
    // num instruments (i)
    k: usize,

    // y -> i -> d -> y's blocked impact by y + 10 + d.
    blk_pos: Vec<Vec<Vec<i32>>>,
    // y -> i -> d -> y's blocked impact by y - 10 - d.
    blk_neg: Vec<Vec<Vec<i32>>>,

    // y -> d -> best score by back musician between y and y + 10 + d.
    add: Vec<Vec<i32>>,

    // y -> i -> total impacts on 180 degrees
    all: Vec<Vec<i32>>,
}

type P = Vector<f64>;

impl Dp {
    pub fn new(problem_id: u32, problem: Problem) -> Self {
        let board = Board::new(problem_id, problem.clone(), SOLVER_NAME);

        let h = board.prob.stage.max.y as usize;
        let k = board.prob.attendees[0].tastes.len();

        let blk_pos = vec![vec![vec![0; 11]; k]; h + 1];
        let blk_neg = vec![vec![vec![0; 11]; k]; h + 1];
        let add = vec![vec![0; 11]; h + 1];
        let all = vec![vec![0; k]; h + 1];

        Self {
            h,
            k,
            blk_pos,
            blk_neg,
            add,
            all,
            orig_problem: problem,
            board,
        }
    }

    // pub fn solve(&mut self) -> (f64, Board) {}
}
