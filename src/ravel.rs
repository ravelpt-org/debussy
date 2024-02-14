use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::Languages;


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

pub async fn get_submissions(creds: &HashMap<&str, String>, client: &Client, url: &String) -> Result<Vec<Submission>, &'static str>  {
	let res = client
		.get(format!("{}/judge/pending", url))
		.header("Content-Type", "application/json")
		.json(&creds)
		.send()
		.await
		.unwrap();

	match res.status() {
		reqwest::StatusCode::OK => return match res.json::<Submissions>().await {
			Ok(parsed) => Ok(parsed.submissions),
			Err(err) => {
				println!("{:?}", err);
				Err("Unable to get submissions. Response did not match type of Input.")
			},
		},
		_other => Err("Unable to get submissions. Response was not ok")
	}
}