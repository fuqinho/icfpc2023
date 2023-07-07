use anyhow::Result;
use rand::Rng;

use tanakh_solver::api::{get_problem, submit, Placement};

fn main() -> Result<()> {
    let problem_id = 42;

    let problem = get_problem(problem_id)?;

    let n = problem.musicians.len();
    let mut placements = vec![];

    let stage_x = problem.stage_bottom_left[0];
    let stage_y = problem.stage_bottom_left[1];

    for _ in 0..n {
        let x = rand::thread_rng().gen_range(stage_x + 10.0..stage_x + problem.stage_width - 10.0);
        let y = rand::thread_rng().gen_range(stage_y + 10.0..stage_y + problem.stage_height - 10.0);
        placements.push(Placement { x, y });
    }

    let id = submit(problem_id, &placements)?;
    eprintln!("{id:?}");

    Ok(())
}
