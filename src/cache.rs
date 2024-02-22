use crate::error::Errors;
use anyhow::anyhow;
use anyhow::{Context, Result};
use md5;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Problem {
    problem_input: String,
    problem_output: String,
}

pub async fn check_cache(problem_id: &i32, input_sum: String, output_sum: String) -> Result<bool> {
    if !Path::exists(Path::new(&format!("problems/{}", problem_id))) {
        return Ok(false);
    }

    let input = fs::read_to_string(format!("problems/{}/input.txt", problem_id))
        .await
        .with_context(|| format!("Failed to read problem {}'s input.", problem_id))?;
    let output = fs::read_to_string(format!("problems/{}/output.txt", problem_id))
        .await
        .with_context(|| format!("Failed to read problem {}'s output.", problem_id))?;

    let _input2 = input.clone();

    if format!("{:x}", md5::compute(input)) != input_sum
        || format!("{:x}", md5::compute(output)) != output_sum
    {
        return Ok(false);
    }

    return Ok(true);
}

pub async fn cache_problem(
    creds: &HashMap<&str, String>,
    client: &reqwest::Client,
    url: &String,
    problem_id: i32,
) -> Result<()> {
    if Path::exists(Path::new(&format!("problems/{}", problem_id))) {
        fs::remove_dir_all(&format!("problems/{}", problem_id))
            .await
            .with_context(|| format!("Unable to clear dir for problem {}.", problem_id))?;
    }
    fs::create_dir(&format!("problems/{}", problem_id))
        .await
        .with_context(|| format!("Unable to create dir for problem {}.", problem_id))?;

    let mut json = creds.clone();
    json.insert("problem", problem_id.to_string());
    let res = client
        .get(format!("{}/judge/problem", url))
        .header("Content-Type", "application/json")
        .json(&json)
        .send()
        .await?;

    return match res.status() {
        reqwest::StatusCode::OK => match res.json::<Problem>().await {
            Ok(parsed) => {
                fs::write(
                    format!("problems/{}/input.txt", problem_id),
                    &parsed.problem_input,
                )
                .await
                .with_context(|| format!("Unable to write input for problem {}.", problem_id))?;
                fs::write(
                    format!("problems/{}/output.txt", problem_id),
                    &parsed.problem_output,
                )
                .await
                .with_context(|| format!("Unable to write output for problem {}.", problem_id))?;
                Ok(())
            }
            Err(_) => return Err(anyhow!(Errors::ProblemFetchError)),
        },
        _other => Err(anyhow!(Errors::RavelError)),
    };
}
