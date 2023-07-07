use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct Problem {
    room_width: f64,
    room_height: f64,
    stage_width: f64,
    stage_height: f64,
    stage_bottom_left: Vec<f64>,
    musicians: Vec<usize>,
    attendees: Vec<Atendee>,
}

#[derive(Deserialize, Debug)]
struct Atendee {
    x: f64,
    y: f64,
    tastes: Vec<f64>,
}

#[derive(Serialize)]
struct Solution {
    placements: Vec<Placement>,
}

#[derive(Serialize)]
struct Placement {
    x: f64,
    y: f64,
}

#[derive(Deserialize, Debug)]
struct Response {
    #[serde(rename = "Success")]
    success: String,
}

const ENDPOINT: &str = "https://api.icfpcontest.com";

fn get_problem(problem_id: u32) -> Result<Problem> {
    let url = format!("{ENDPOINT}/problem?problem_id={problem_id}");
    let resp: Response = reqwest::blocking::get(&url)?.json()?;
    let problem: Problem = serde_json::from_str(&resp.success)?;
    Ok(problem)
}

fn main() -> Result<()> {
    let problem = get_problem(1)?;
    println!("{:?}", problem);
    Ok(())
}
