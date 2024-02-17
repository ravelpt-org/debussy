use crate::cache;
use crate::ravel::Submission;
use anyhow::{Context, Result};
use docker_api;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum JobStatus {
    Pending,
    Running,
    Finished,
}

pub async fn run_submission(
    submission: Submission,
    client: &reqwest::Client,
    creds: &HashMap<&str, String>,
    url: &String,
    docker: &docker_api::docker::Docker,
) -> Result<()> {
    // Cache problem info
    match cache::check_cache(
        &submission.problem,
        submission.input_sum,
        submission.output_sum,
    )
    .await?
    {
        false => {
            println!("Problem {} is missing from cache", submission.problem);
            cache::cache_problem(creds, client, url, submission.problem).await?;
        }
        _ => {}
    }

    //fs::create_dir(format!("./jobs/{}", submission.id)).await.unwrap();

    let opts = docker_api::opts::ContainerCreateOpts::builder()
        .auto_remove(true)
        .name(format!("reverie_{}", submission.id))
        .volumes("debussy-sandbox".chars())
        .network_mode("none")
        .image("reverie")
        .build();

    match docker.containers().create(&opts).await {
        Ok(info) => println!("Ok: {info:?}"),
        Err(e) => eprintln!("Error: {e}"),
    };

    //docker.containers().create(&opts).await.with_context(|| format!("Unable to create container for submission {}", submission.id))?;

    Ok(())
}
