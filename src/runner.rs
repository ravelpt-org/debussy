use crate::cache;
use crate::docker::{create_container, ContainerOptions};
use crate::ravel::Submission;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
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

    if !Path::exists(Path::new(&format!("./jobs/{}", submission.id))) {
        fs::create_dir(format!("./jobs/{}", submission.id))
            .await
            .unwrap();
    }

    let mut volumes = HashMap::new();
    let mut volume_mounts = HashMap::new();
    volume_mounts.insert("/jobs/291".to_string(), "debussy-sandbox".to_string()); // Docker volume named "debussy-sandbox"
    volumes.insert("/usr/src".to_string(), volume_mounts); // Mapping debussy-sandbox volume to /usr/src in the container

    let container_options = ContainerOptions {
        image: "reverie".to_string(),
        host_config: crate::docker::HostConfig {
            binds: None,
            auto_remove: true,
        },
        tty: true,
        attach_stdin: false,
        attach_stdout: true,
        attach_stderr: true,
        open_stdin: false,
        stdin_once: false,
        env: None,
        volumes: Some(volumes),
    };

    create_container(
        container_options,
        format!("reverie_{}", submission.id),
        String::from("http://localhost:2375"),
    )
    .await?;

    Ok(())
}
