mod ravel;
mod cache;
mod runner;
mod error;

use std::collections::HashMap;
use dotenvy;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use crate::runner::run_submission;

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

	loop {
		let subs = ravel::get_submissions(&ravel_creds, &client, &url).await.expect("Unable to get submissions");
		for sub in subs {
			let _ = run_submission(sub, &client, &ravel_creds, &url).await;
		}
	}
}
