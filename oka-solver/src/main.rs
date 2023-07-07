use anyhow::Result;
use common::{board::Board, evaluate, Problem, Solution};
use lyon_geom::Point;
use rand::{rngs::StdRng, Rng, SeedableRng};

#[argopt::cmd]
fn main(
    /// time limit in seconds
    #[opt(long, default_value = "10.0")]
    time_limit: f64,
    /// number of threads
    #[opt(long, default_value = "1")]
    threads: usize,
    /// problem id
    problem_id: u32,
) -> Result<()> {
    let mut rng = StdRng::seed_from_u64(42);
    // let mut rng = rand::thread_rng();

    let problem = Problem::read_from_file(format!("problems/{}.json", problem_id))?;

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

    println!("score: {}", board.score());

    let solution: Solution = board.clone().try_into().unwrap();

    let actual_score = evaluate(&problem, &solution);
    println!("actual score: {}", actual_score);

    for i in 0..problem.musicians.len() {
        board.unplace(i);
    }

    assert_eq!(board.score(), 0.0);

    Ok(())
}
