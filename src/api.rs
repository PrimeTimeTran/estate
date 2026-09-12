use crate::{model::problem_model::StoredProblem, prelude::*, proto::types::SampleProblemRequest};

impl Clone for Box<dyn Api> {
	fn clone(&self) -> Self {
		self.clone_box()
	}
}
