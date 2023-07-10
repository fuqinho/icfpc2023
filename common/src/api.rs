use std::convert::TryFrom;

use anyhow::{bail, Ok, Result};
use serde::Deserialize;
use serde_json::json;

use crate::problem::{Problem, RawProblem, RawSolution, Solution};

const ENDPOINT: &str = "https://api.icfpcontest.com";

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum Score {
    Success(f64),
    Failure(String),
    Processing,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Submission {
    pub _id: String,
    pub problem_id: u32,
    pub user_id: String,
    pub score: Score,
    pub submitted_at: String,
}

#[derive(Deserialize, Clone, Debug)]
struct RawSubmissionEntry {
    submission: Submission,
    contents: String,
}

#[derive(Clone, Debug)]
pub struct SubmissionEntry {
    pub submission: Submission,
    pub contents: Solution,
}

impl TryFrom<RawSubmissionEntry> for SubmissionEntry {
    type Error = anyhow::Error;
    fn try_from(raw: RawSubmissionEntry) -> Result<Self> {
        Ok(Self {
            submission: raw.submission,
            contents: Solution::from(serde_json::from_str::<RawSolution>(&raw.contents)?),
        })
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ScoreboardEntry {
    pub username: String,
    pub score: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Scoreboard {
    pub frozen: bool,
    pub scoreboard: Vec<ScoreboardEntry>,
    pub updated_at: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Userboard {
    pub problems: Vec<Option<f64>>,
}

#[derive(Clone, Debug)]
pub struct Token(String);

#[derive(Clone, Debug)]
pub struct SubmissionId(pub String);

pub struct Client {
    client: reqwest::blocking::Client,
    token: Option<Token>,
}

#[derive(Deserialize, Clone, Debug)]
struct NumProblems {
    number_of_problems: u32,
}

#[derive(Deserialize, Clone, Debug)]
enum Response<T> {
    Success(T),
    Failure(String), // message
}

impl Client {
    pub fn new() -> Self {
        let token: Option<Token> = std::env::var("API_TOKEN").map(|s| Token(s)).ok();
        Self {
            client: reqwest::blocking::Client::new(),
            token,
        }
    }

    pub fn get_submission(&self, id: &str) -> Result<SubmissionEntry> {
        let url = format!("{ENDPOINT}/submission?submission_id={id}");
        let res: Response<RawSubmissionEntry> = self
            .client
            .get(&url)
            .bearer_auth(&self.token.as_ref().expect("API_TOKEN is missing").0)
            .send()?
            .json()?;
        match res {
            Response::Success(s) => Ok(SubmissionEntry::try_from(s)?),
            Response::Failure(s) => bail!(s),
        }
    }

    pub fn post_submission(&self, problem_id: u32, s: Solution) -> Result<SubmissionId> {
        let url = format!("{ENDPOINT}/submission");
        let res = self
            .client
            .post(&url)
            .bearer_auth(&self.token.as_ref().expect("API_TOKEN is missing").0)
            .json(&json!({
                "problem_id": problem_id,
                "contents": serde_json::to_string(&RawSolution::from(s))?,
            }))
            .send()?
            .text()?;
        Ok(SubmissionId(res))
    }

    pub fn get_submissions(
        &self,
        offset: u64,
        limit: i64,
        problem_id: Option<u32>,
    ) -> Result<Vec<Submission>> {
        let mut url = format!("{ENDPOINT}/submissions?offset={offset}&limit={limit}");
        if let Some(problem_id) = problem_id {
            url += &format!("&problem_id={problem_id}");
        }
        let res: Response<Vec<Submission>> = self
            .client
            .get(&url)
            .bearer_auth(&self.token.as_ref().expect("API_TOKEN is missing").0)
            .send()?
            .json()?;
        match res {
            Response::Success(ss) => Ok(ss),
            Response::Failure(s) => bail!(s),
        }
    }

    pub fn get_problem(&self, problem_id: u32) -> Result<Problem> {
        let url = format!("https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/problems/{problem_id}/spec");
        let raw_problem: RawProblem = self.client.get(&url).send()?.json()?;
        Ok(Problem::from(raw_problem))
    }

    pub fn get_problems(&self) -> Result<u32> {
        let url = format!("{ENDPOINT}/problems");
        let res: NumProblems = self.client.get(&url).send()?.json()?;
        Ok(res.number_of_problems)
    }

    pub fn get_scoreboard(&self) -> Result<Scoreboard> {
        let url = format!("{ENDPOINT}/scoreboard");
        let res: Scoreboard = self.client.get(&url).send()?.json()?;
        Ok(res)
    }

    pub fn get_userboard(&self) -> Result<Userboard> {
        let url = format!("{ENDPOINT}/userboard");
        let res: Response<Userboard> = self
            .client
            .get(&url)
            .bearer_auth(&self.token.as_ref().expect("API_TOKEN is missing").0)
            .send()?
            .json()?;
        match res {
            Response::Success(u) => Ok(u),
            Response::Failure(s) => bail!(s),
        }
    }

    // Returns access token.
    pub fn post_register(&self, username: &str, email: &str, password: &str) -> Result<Token> {
        let url = format!("{ENDPOINT}/register");
        let res: Response<String> = self
            .client
            .post(&url)
            .json(&json!({
                "username": username,
                "email": email,
                "password": password,
            }))
            .send()?
            .json()?;
        match res {
            Response::Success(s) => Ok(Token(s)),
            Response::Failure(s) => bail!(s),
        }
    }

    // Returns access token.
    pub fn post_login(&self, username_or_email: &str, password: &str) -> Result<Token> {
        let url = format!("{ENDPOINT}/login");
        let res: Response<String> = self
            .client
            .post(&url)
            .json(&json!({
                "username_or_email": username_or_email,
                "password": password,
            }))
            .send()?
            .json()?;
        match res {
            Response::Success(s) => Ok(Token(s)),
            Response::Failure(s) => bail!(s),
        }
    }
}

pub fn get_best_solution(problem_id: u32) -> Result<Solution> {
    let url = format!(
        "https://icfpc2023-backend-uadsges7eq-an.a.run.app/api/problems/{problem_id}/best-solution"
    );
    let raw: RawSolution = reqwest::blocking::get(&url)?.json()?;
    Ok(raw.into())
}
