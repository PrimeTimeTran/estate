use tonic::{Request, Response, Status};

use crate::{
	prelude::*,
	proto::leetcode::{
		CreateSubmissionRequest, DeleteSubmissionRequest, Empty, GetSubmissionRequest,
		ListSubmissionsRequest, ListSubmissionsResponse, RunSubmissionRequest, Submission,
		SubmissionStatus, UpdateSubmissionRequest,
	},
	repo::submission::{CreateSubmission, SubmissionQuery, SubmissionRepository, UpdateSubmission},
};

#[derive(Default)]
pub struct SubmissionServiceImpl<R> {
	repository: R,
}

impl<R> SubmissionServiceImpl<R> {
	pub fn new(repository: R) -> Self {
		Self { repository }
	}
}

#[tonic::async_trait]
impl<R> crate::proto::leetcode::submission_service_server::SubmissionService
	for SubmissionServiceImpl<R>
where
	R: SubmissionRepository + Send + Sync + 'static,
{
	async fn list_submissions(
		&self,
		request: Request<ListSubmissionsRequest>,
	) -> Result<Response<ListSubmissionsResponse>, Status> {
		let request = request.into_inner();
		let page = request.page.unwrap();

		let query = SubmissionQuery {
			page: Some(page.page),
			page_size: Some(page.page_size),
			user_id: request.user_id,
			problem_id: request.problem_id,
			status: submission_status(request.status)?,
			language: request.language,
		};

		let result = self.repository.list(query).await.map_err(internal_error)?;

		Ok(Response::new(ListSubmissionsResponse {
			submissions: result.items,
			page: Some(crate::proto::leetcode::PageInfo {
				page: result.page as i32,
				page_size: result.page_size as i32,
				total: result.total as i64,
			}),
		}))
	}

	async fn create_submission(
		&self,
		request: Request<CreateSubmissionRequest>,
	) -> Result<Response<Submission>, Status> {
		let request = request.into_inner();

		let user_id = request
			.user_id
			.parse::<i64>()
			.map_err(|_| Status::invalid_argument("invalid user id"))?;

		let problem_id = request
			.problem_id
			.parse::<i64>()
			.map_err(|_| Status::invalid_argument("invalid problem id"))?;

		let submission = CreateSubmission {
			user_id: request.user_id,
			problem_id: request.problem_id,
			source: request.source,
			language: request.language,
		};

		let submission = self
			.repository
			.create(submission)
			.await
			.map_err(internal_error)?;

		Ok(Response::new(submission))
	}

	async fn update_submission(
		&self,
		request: Request<UpdateSubmissionRequest>,
	) -> Result<Response<Submission>, Status> {
		let request = request.into_inner();

		let submission = UpdateSubmission {
			source: request.source,
			language: request.language,
		};

		let submission = self
			.repository
			.update(&request.id, submission)
			.await
			.map_err(internal_error)?;

		Ok(Response::new(submission))
	}

	async fn delete_submission(
		&self,
		request: Request<DeleteSubmissionRequest>,
	) -> Result<Response<Empty>, Status> {
		let request = request.into_inner();

		self
			.repository
			.delete(&request.id)
			.await
			.map_err(internal_error)?;

		Ok(Response::new(Empty {}))
	}

	async fn get_submission(
		&self,
		request: Request<GetSubmissionRequest>,
	) -> Result<Response<Submission>, Status> {
		let request = request.into_inner();

		let submission = self
			.repository
			.get(&request.id)
			.await
			.map_err(internal_error)?;

		Ok(Response::new(submission))
	}

	async fn run_submission(
		&self,
		request: Request<RunSubmissionRequest>,
	) -> Result<Response<Submission>, Status> {
		Err(Status::unimplemented("run_submission is not implemented"))
	}
}

fn internal_error(error: anyhow::Error) -> Status {
	tracing::error!("{error:#}");
	Status::internal(error.to_string())
}

fn submission_status(status: Option<i32>) -> Result<Option<SubmissionStatus>, Status> {
	status
		.map(SubmissionStatus::try_from)
		.transpose()
		.map_err(|_| Status::invalid_argument("invalid submission status"))
}
