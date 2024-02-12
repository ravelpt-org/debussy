mod ravel;

use std::collections::HashMap;
use dotenvy;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

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
	if Path::exists(Path::new("problems/")) {
		fs::remove_dir_all("problems/").expect("Unable to clear problems directory");
	}
	fs::create_dir("problems/").expect("Unable to create problems directory");

	let client = reqwest::Client::builder().build().unwrap();

	loop {
		println!("{:?}", ravel::get_submissions(&ravel_creds, &client, &url).await.expect("Unable to get submissions"));
	}
}
