use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::Languages;


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Submission {
	// Submission id
	id: i32,
	// Time code is in
	language: Languages,
	// Time submitted
	time: String,
	// Code
	content: String,
	// Problem id
	problem: i32,
	// Input file checksum
	input_sum: String,
	// Output file checksum
	output_sum: String,
	// Timelimit
	timeout: i32,
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