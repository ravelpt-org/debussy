use std::collections::HashMap;
use crate::ravel::Submission;
use crate::cache;
use anyhow::Result;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum JobStatus {
	Pending,
	Running,
	Finished
}

pub async fn run_submission(submission: Submission, client: &reqwest::Client, creds: &HashMap<&str, String>, url: &String) -> Result<()> {
	// Cache problem info
	match cache::check_cache(&submission.problem, submission.input_sum, submission.output_sum).await? {
		false => {
			println!("Problem {} is missing from cache", submission.problem);
			cache::cache_problem(creds, client, url, submission.problem).await?;
		},
		_ => {}
	}

	Ok(())
}