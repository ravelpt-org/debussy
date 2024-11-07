use crate::docker::{create_container, start_container, ContainerOptions};
use crate::ravel::Submission;
use crate::{cache, Languages};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum JobStatus {
    Pending,
    Running,
    Finished,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum JobResult {
    Correct,
    Wrong,
    TimelimitException,
    RuntimeError,
    CompilerError,
    IllegalImport,
}

impl JobResult {
    pub fn from_i32(i: i32) -> Option<Self> {
        match i {
            1 => Some(Self::Correct),
            2 => Some(Self::Wrong),
            3 => Some(Self::TimelimitException),
            4 => Some(Self::RuntimeError),
            5 => Some(Self::CompilerError),
            6 => Some(Self::IllegalImport),
            _ => None,
        }
    }
}

pub async fn run_submission(
    submission: Submission,
    client: &reqwest::Client,
    creds: &HashMap<&str, String>,
    url: &String,
) -> Result<()> {
    // Cache problem info
    match cache::check_cache(
        &submission.problem,
        submission.input_sum,
        submission.output_sum,
    )
    .await
    {
        Ok(false) => {
            info!("Problem {} is missing from cache", submission.problem);
            cache::cache_problem(creds, client, url, submission.problem).await?;
        }
        Err(_) => {
            error!("Unable to read problem {} from cache", submission.problem);
            cache::cache_problem(creds, client, url, submission.problem).await?;
        }
        _ => {}
    }

    if Path::exists(Path::new(&format!("./jobs/{}", submission.id))) {
        fs::remove_dir_all(format!("./jobs/{}", submission.id))
            .await
            .with_context(|| {
                format!(
                    "Unable to remove existing dir for submission {}",
                    submission.id
                )
            })?;
    }
    // TODO: Lock dir once files are written
    fs::create_dir(format!("./jobs/{}", submission.id))
        .await
        .with_context(|| format!("Unable to create dir for submission {}", submission.id))?;
    fs::copy(
        format!("./problems/{}/input.txt", submission.problem),
        format!("./jobs/{}/input.txt", submission.id),
    )
    .await
    .with_context(|| format!("Unable to copy input for submission {}", submission.id))?;
    fs::copy(
        format!("./problems/{}/output.txt", submission.problem),
        format!("./jobs/{}/output.txt", submission.id),
    )
    .await
    .with_context(|| format!("Unable to copy input for submission {}", submission.id))?;
    match submission.language {
        Languages::Python => fs::write(
            format!("./jobs/{}/solution.py", submission.id),
            submission.content,
        ),
        Languages::Java => fs::write(
            format!("./jobs/{}/solution.java", submission.id),
            submission.content,
        ),
        Languages::Cpp => fs::write(
            format!("./jobs/{}/solution.cpp", submission.id),
            submission.content,
        ),
    }
    .await?;

    let mut binds = Vec::new();
    binds.push(format!(
        "{}/jobs/{}:/usr/src/debussy",
        env::current_dir()?.display(),
        submission.id
    ));
    let mut env = Vec::new();
    env.push(format!("TIMEOUT={}", submission.timeout));

    let container_options = ContainerOptions {
        image: "reverie_test".to_string(),
        //image: "ghcr.io/timbercreekprogrammingteam/reverie:latest".to_string(),
        host_config: crate::docker::HostConfig {
            binds: Some(binds),
            auto_remove: false,
        },
        tty: true,
        attach_stdin: true,
        attach_stdout: true,
        attach_stderr: true,
        open_stdin: true,
        stdin_once: false,
        network_disabled: true,
        env: Some(env),
        volumes: None,
    };

    create_container(
        container_options,
        format!("reverie_{}", submission.id),
        String::from("http://localhost:2375"),
    )
    .await?;

    debug!(
        "Container for submission '{}', has been created",
        submission.id
    );

    start_container(
        format!("reverie_{}", submission.id),
        String::from("http://localhost:2375"),
    )
    .await?;

    debug!(
        "Container for submission '{}', has been started",
        submission.id
    );

    Ok(())
}
