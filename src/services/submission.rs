use crate::services::*;

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
impl<R> SubmissionService for SubmissionServiceImpl<R>
where
	R: SubmissionRepository + Send + Sync + 'static,
{
	async fn list_submissions(
		&self,
		request: Request<ListSubmissionsRequest>,
	) -> Result<Response<ListSubmissionsResponse>, Status> {
		let request = request.into_inner();
		let page = page_request(request.page)?;
		let result = self
			.repository
			.list(SubmissionQuery {
				page: Some(page.page),
				page_size: Some(page.page_size),
				user_id: request.user_id,
				problem_id: request.problem_id,
				status: submission_status(request.status)?,
				language: request.language,
			})
			.await
			.map_err(internal_error)?;
		Ok(Response::new(ListSubmissionsResponse {
			submissions: result.items.clone(),
			page: Some(result.page_info()),
		}))
	}
	async fn create_submission(
		&self,
		request: Request<CreateSubmissionRequest>,
	) -> Result<Response<Submission>, Status> {
		let request = request.into_inner();
		let submission = self
			.repository
			.create(CreateSubmission {
				user_id: request.user_id,
				problem_id: request.problem_id,
				source: request.source,
				language: request.language,
			})
			.await
			.map_err(internal_error)?;
		Ok(Response::new(submission))
	}
	async fn update_submission(
		&self,
		request: Request<UpdateSubmissionRequest>,
	) -> Result<Response<Submission>, Status> {
		let request = request.into_inner();
		let submission = self
			.repository
			.update(
				&request.id,
				UpdateSubmission {
					source: request.source,
					language: request.language,
				},
			)
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
		_request: Request<RunSubmissionRequest>,
	) -> Result<Response<Submission>, Status> {
		Err(Status::unimplemented("run_submission is not implemented"))
	}
}
fn submission_status(status: Option<i32>) -> Result<Option<SubmissionStatus>, Status> {
	status
		.map(SubmissionStatus::try_from)
		.transpose()
		.map_err(|_| Status::invalid_argument("invalid submission status"))
}
