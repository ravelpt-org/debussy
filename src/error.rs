#[derive(Debug)]
pub enum Errors {
	RavelError,
	SubmissionFetchError,
	ProblemFetchError,
}

impl std::fmt::Display for Errors {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::RavelError => write!(f, "Error communicating to ravel"),
			Self::SubmissionFetchError => write!(f, "Unable to fetch submissions from ravel. Response did not match type of input."),
			Self::ProblemFetchError => write!(f, "Unable to fetch problem in/out from ravel. Response did not match type of input."),
		}
	}
}