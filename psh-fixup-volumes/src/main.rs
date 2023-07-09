use anyhow::Result;

use common::api;
use common::problem::{Problem, Solution};
use common::{evaluate, fixup_volumes};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let problem_id: u32 = args[1].parse()?;
    let problem = Problem::read_from_file(format!("./problems/{problem_id}.json"))?;
    let old_solution = if args[2] == "best" {
        api::get_best_solution(problem_id)?
    } else {
        Solution::read_from_file(&args[2])?
    };
    let mut new_solution = fixup_volumes(&problem, &old_solution);
    new_solution.solver += "-fixed-volume";
    let old_score = evaluate(&problem, &old_solution) as i64;
    let new_score = evaluate(&problem, &new_solution) as i64;
    println!("Score: {:} -> {:}", old_score, new_score);
    Solution::write_to_file(
        format!("./psh-solution/{problem_id}-fixed-{new_score}.json"),
        new_solution.clone(),
    )?;
    println!(
        "Submit = {:?}",
        api::Client::new().post_submission(problem_id, new_solution)?
    );
    Ok(())
}
