extern crate common;

use anyhow::Result;

use common::api;
use common::api::Score;
use common::evaluate;
use common::problem::Problem;

fn validate(offset: u64, limit: i64, problem_id: Option<u32>) -> Result<()> {
    let client = api::Client::new();
    let submissions = client.get_submissions(offset, limit, problem_id)?;
    for submission_id in submissions.iter().map(|s| &s._id) {
        let entry = client.get_submission(submission_id)?;
        let problem_id = entry.submission.problem_id;
        let problem = Problem::read_from_file(format!("problems/{problem_id}.json"))?;
        let score = evaluate(&problem, &entry.contents);
        if entry.submission.score != Score::Success(score as f64) {
            println!(
                "Wrong score {submission_id}, {:?}, {:?}",
                entry.submission.score, score
            );
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let problem_id = if args.len() >= 4 {
        Some(args[3].parse::<u32>()?)
    } else {
        None
    };
    validate(args[1].parse::<u64>()?, args[2].parse::<i64>()?, problem_id)?;
    Ok(())
}
