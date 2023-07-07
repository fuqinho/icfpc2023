use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
enum Response<T> {
    Success(T),
    Failure(String),
}

const ENDPOINT: &str = "https://api.icfpcontest.com";

fn get_problem(problem_id: u32) -> Result<Problem> {
    let url = format!("{ENDPOINT}/problem?problem_id={problem_id}");
    let resp: Response<String> = reqwest::blocking::get(&url)?.json()?;
    match resp {
        Response::Success(s) => Ok(serde_json::from_str(&s)?),
        Response::Failure(s) => anyhow::bail!("Failure: {s}"),
    }
}

#[derive(Debug)]
struct SubmitId(String);

fn submit(problem_id: u32, solution: &[Placement]) -> Result<SubmitId> {
    let url = format!("{ENDPOINT}/submission");
    let client = reqwest::blocking::Client::new();
    let token = std::env::var("API_TOKEN")?;
    let resp: String = dbg!(client
        .post(&url)
        .bearer_auth(token)
        .json(&json!({
            "problem_id": problem_id,
            "contents": serde_json::to_string(&json!({
                "placements": solution
            }))?
        }))
        .send()?)
    .text()?;
    Ok(SubmitId(resp))
}

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
