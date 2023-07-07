use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct Problem {
    pub room_width: f64,
    pub room_height: f64,
    pub stage_width: f64,
    pub stage_height: f64,
    pub stage_bottom_left: Vec<f64>,
    pub musicians: Vec<usize>,
    pub attendees: Vec<Atendee>,
}

#[derive(Deserialize, Debug)]
pub struct Atendee {
    pub x: f64,
    pub y: f64,
    pub tastes: Vec<f64>,
}

#[derive(Serialize)]
pub struct Solution {
    pub placements: Vec<Placement>,
}

#[derive(Serialize)]
pub struct Placement {
    pub x: f64,
    pub y: f64,
}

#[derive(Deserialize, Debug)]
pub enum Response<T> {
    Success(T),
    Failure(String),
}

const ENDPOINT: &str = "https://api.icfpcontest.com";

pub fn get_problem(problem_id: u32) -> Result<Problem> {
    let url = format!("{ENDPOINT}/problem?problem_id={problem_id}");
    let resp: Response<String> = reqwest::blocking::get(&url)?.json()?;
    match resp {
        Response::Success(s) => Ok(serde_json::from_str(&s)?),
        Response::Failure(s) => anyhow::bail!("Failure: {s}"),
    }
}

#[derive(Debug)]
pub struct SubmitId(pub String);

pub fn submit(problem_id: u32, solution: &[Placement]) -> Result<SubmitId> {
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
