mod cache;
mod docker;
mod error;
mod ravel;
mod runner;

use crate::runner::JobResult::Correct;
use crate::runner::{run_submission, JobStatus};
use chrono::Utc;
use dotenvy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing;
use tracing::{error, info};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Languages {
    Python,
    Java,
    Cpp,
}

struct Job {
    submission: ravel::Submission,
    span: tracing::Span,
    status: JobStatus,
}

#[tokio::main]
async fn main() {
    // Init env vars
    dotenvy::dotenv().expect("Dotenvy not initialized");
    let url = dotenvy::var("ravel_url").expect("No ravel_url set in .env");
    let max_jobs = dotenvy::var("max_jobs")
        .expect("No max_jobs sent in .env")
        .parse()
        .expect("max_jobs should be and int");

    let mut ravel_creds = HashMap::new();
    ravel_creds.insert(
        "username",
        dotenvy::var("ravel_username").expect("No username var"),
    );
    ravel_creds.insert(
        "password",
        dotenvy::var("ravel_password").expect("No username var"),
    );

    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set subscribe as defualt");

    // Init problem dir
    if !Path::exists(Path::new("problems/")) {
        fs::create_dir("problems/").expect("Unable to create problems directory");
    }

    // Init jobs dir
    if Path::exists(Path::new("jobs/")) {
        fs::remove_dir_all("jobs/").expect("Unable to clear jobs directory");
    }
    fs::create_dir("jobs/").expect("Unable to create jobs directory");

    let client = reqwest::Client::builder().build().unwrap();

    let mut jobs = HashMap::new();
    let mut timestamp = Utc::now().time();
    let mut num_running_jobs = 0;
    let mut finished = ravel::Update {
        username: ravel_creds.get("username").unwrap().to_owned(),
        password: ravel_creds.get("password").unwrap().to_owned(),
        submissions: Vec::new(),
    };

    loop {
        // Process submissions from Ravel
        if (Utc::now().time() - timestamp).num_seconds() >= 5 {
            timestamp = Utc::now().time();

            for sub in ravel::get_submissions(&ravel_creds, &client, &url)
                .await
                .expect("Unable to get submissions")
            {
                if !jobs.contains_key(&sub.id) {
                    jobs.insert(
                        sub.id,
                        Job {
                            span: tracing::span!(tracing::Level::TRACE, "Submission", id = sub.id,),
                            submission: sub,
                            status: JobStatus::Pending,
                        },
                    );
                }
            }
        }

        for job in jobs.values_mut() {
            match job.status {
                JobStatus::Pending => {
                    if num_running_jobs <= max_jobs {
                        let _enter = job.span.enter();
                        info!("Running submission '{}'", job.submission.id);
                        match run_submission(job.submission.clone(), &client, &ravel_creds, &url)
                            .await
                        {
                            Ok(_) => {
                                num_running_jobs += 1;
                                job.status = JobStatus::Running;
                                info!("Judging submission '{}', has started", job.submission.id)
                            }
                            Err(err) => {
                                error!(
                                    "Encountered an error running submission '{}': '{}'",
                                    job.submission.id, err
                                );
                            }
                        }
                    }
                }
                JobStatus::Running => {
                    if Path::exists(&Path::new(&format!(
                        "./jobs/{}/status.txt",
                        job.submission.id
                    ))) {
                        job.status = JobStatus::Finished;
                        num_running_jobs -= 1;
                    }
                }
                JobStatus::Finished => {
                    let _enter = job.span.enter();
                    info!("Submission '{}' has finished running", job.submission.id);

                    let result = runner::JobResult::from_string(
                        &tokio::fs::read_to_string(format!(
                            "./jobs/{}/status.txt",
                            job.submission.id
                        ))
                        .await
                        .unwrap(),
                    );
                    if result == None {
                        job.status = JobStatus::Pending;
                        error!(
                            "Error judging submission '{}', status returned None",
                            job.submission.id
                        );
                        continue;
                    }

                    let mut solved = true;
                    let mut err = None;
                    if result != Some(Correct) {
                        solved = false;
                        err = result;
                    }

                    info!(
                        "Submission '{}' has finished with the result solved: '{}', err: '{:?}'",
                        job.submission.id, solved, err
                    );

                    if solved == false {
                        error!(
                            "Submission '{}' ended with error '{:?}'",
                            job.submission.id,
                            tokio::fs::read_to_string(format!(
                                "./jobs/{}/error.txt",
                                job.submission.id
                            ))
                            .await
                        );
                    }

                    finished.submissions.push(ravel::FinishedSubmissions {
                        id: job.submission.id,
                        solved,
                        error: err,
                    });
                }
            }
        }

        if finished.submissions.len() > 0 {
            match client
                .post(format!("{}/judge/update", url))
                .json(&finished)
                .send()
                .await
            {
                Ok(_) => {
                    for job in &finished.submissions {
                        let _ = tokio::fs::remove_dir_all(format!("./jobs/{}", job.id)).await;
                        jobs.remove(&job.id);
                    }
                    finished.submissions.clear();
                }
                Err(_) => {
                    println!("Unable to update submissions on ravel.");
                }
            }
        }
    }
}
