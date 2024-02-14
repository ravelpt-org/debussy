use crate::ravel::Submission;
use crate::cache;
use anyhow::Result;

pub async fn run_submission(submission: Submission) -> Result<()> {
	// Cache problem info
	match cache::check_cache(submission.problem, submission.input_sum, submission.output_sum).await? {
		false => println!("Problem {} is not cached, or needs to be updated.", submission.problem),
		_ => {}
	}

	Ok(())
}