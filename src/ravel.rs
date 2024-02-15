use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::Languages;
use anyhow::{anyhow, Context, Result};
use crate::error::Errors;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Submission {
	// Submission id
	pub id: i32,
	// Time code is in
	pub language: Languages,
	// Time submitted
	pub time: String,
	// Code
	pub content: String,
	// Problem id
	pub problem: i32,
	// Input file checksum
	pub input_sum: String,
	// Output file checksum
	pub output_sum: String,
	// Timelimit
	pub timeout: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Submissions {
	submissions: Vec<Submission>
}

pub async fn get_submissions(creds: &HashMap<&str, String>, client: &Client, url: &String) -> Result<Vec<Submission>>  {
	let res = client
		.get(format!("{}/judge/pending", url))
		.header("Content-Type", "application/json")
		.json(&creds)
		.send()
		.await
		.unwrap();

	return match res.status() {
		reqwest::StatusCode::OK => match res.json::<Submissions>().await {
			Ok(parsed) => Ok(parsed.submissions),
			Err(_) => return Err(anyhow!(Errors::SubmissionFetchError)).context("Error parsing submissions"),
		},
		_other => Err(anyhow!(Errors::RavelError)).context("Unable to retrieve submissions")
	}
}
