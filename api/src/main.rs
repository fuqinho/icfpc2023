extern crate common;

use anyhow::Result;
use common::{api, RawSolution, Solution};
use serde_json;
use std::convert::From;

fn main() -> Result<()> {
    let client = api::Client::new();
    let args: Vec<String> = std::env::args().collect();
    match &*args[1] {
        "submission" => println!("{:?}", client.get_submission(&args[2])),
        "submissions" => {
            let problem_id = if args.len() >= 5 {
                Some(args[4].parse::<u32>()?)
            } else {
                None
            };
            println!(
                "{:?}",
                client.get_submissions(
                    args[2].parse::<u64>()?,
                    args[3].parse::<i64>()?,
                    problem_id
                )
            );
        }
        "submit" => {
            let problem_id = args[2].parse::<u32>()?;
            let solution: common::problem::Solution = {
                let content = std::fs::read_to_string(&args[3])?;
                let raw: RawSolution = serde_json::from_str(&content)?;
                Solution::from(raw)
            };
            println!("{:?}", client.post_submission(problem_id, solution));
        }
        "problem" => println!("{:?}", client.get_problem(args[2].parse::<u32>()?)),
        "problems" => println!("{:?}", client.get_problems()),
        "scoreboard" => println!("{:?}", client.get_scoreboard()),
        "userboard" => println!("{:?}", client.get_userboard()),
        "register" => println!("{:?}", client.post_register(&args[2], &args[3], &args[4])),
        "login" => println!("{:?}", client.post_login(&args[2], &args[3])),
        _ => println!("unknown command"),
    };
    Ok(())
}
