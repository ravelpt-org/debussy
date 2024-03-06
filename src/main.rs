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

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Languages {
    Python,
    Java,
    Cpp,
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
                    jobs.insert(sub.id, (sub, JobStatus::Pending));
                }
            }
        }

        for sub in jobs.values_mut() {
            println!("{} {:?}", sub.0.id, sub.1);
            match sub.1 {
                JobStatus::Pending => {
                    if num_running_jobs <= max_jobs {
                        println!("Running {}", sub.0.id);
                        match run_submission(sub.0.clone(), &client, &ravel_creds, &url).await {
                            Ok(_) => {
                                num_running_jobs += 1;
                                sub.1 = JobStatus::Running
                            }
                            Err(err) => {
                                println!("{}", err);
                            }
                        }
                    }
                }
                JobStatus::Running => {
                    if Path::exists(&Path::new(&format!("./jobs/{}/status.txt", sub.0.id))) {
                        sub.1 = JobStatus::Finished;
                        num_running_jobs -= 1;
                    }
                }
                JobStatus::Finished => {
                    let result = runner::JobResult::from_string(
                        &tokio::fs::read_to_string(format!("./jobs/{}/status.txt", sub.0.id))
                            .await
                            .unwrap(),
                    );
                    if result == None {
                        sub.1 = JobStatus::Pending;
                        println!("Error judging submission {}", sub.0.id);
                        continue;
                    }

                    let mut solved = true;
                    let mut err = None;
                    if result != Some(Correct) {
                        solved = false;
                        err = result;
                    }

                    finished.submissions.push(ravel::FinishedSubmissions {
                        id: sub.0.id,
                        solved,
                        error: err,
                    });

                    println!(
                        "Container '{}' finished successfully, with result: {:?}",
                        sub.0.id, result
                    );
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
