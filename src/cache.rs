use std::path::Path;
use tokio::fs;
use md5;
use anyhow::{Context, Result};

pub async fn check_cache(problem_id: i32, input_sum: String, output_sum: String) -> Result<bool> {
	if !Path::exists(Path::new(&format!("problems/{}", problem_id))) {
		return Ok(false);
	}

	let input = fs::read_to_string(format!("problems/{}/input.txt", problem_id))
		.await.with_context(|| format!("Failed to read problem {}'s input.", problem_id))?;
	let output = fs::read_to_string(format!("problems/{}/output.txt", problem_id))
		.await.with_context(|| format!("Failed to read problem {}'s output.", problem_id))?;

	if format!("{:x}", md5::compute(input)) != input_sum
		|| format!("{:x}", md5::compute(output)) != output_sum
	{
		println!("Problem {} is missing from the cache", problem_id);
		return Ok(false);
	}

	return Ok(true);
}