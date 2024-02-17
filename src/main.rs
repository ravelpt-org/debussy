mod cache;
mod docker;
mod error;
mod ravel;
mod runner;

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

        // TODO: Move if to inside match for pending
        if jobs.len() < max_jobs {
            for mut sub in jobs.values_mut() {
                match sub.1 {
                    JobStatus::Pending => {
                        println!("Running {}", sub.0.id);
                        if run_submission(sub.0.clone(), &client, &ravel_creds, &url)
                            .await
                            .is_ok()
                        {
                            sub.1 = JobStatus::Running;
                        }
                    }
                    JobStatus::Running => {}
                    JobStatus::Finished => {}
                }
            }
        }
    }
}
