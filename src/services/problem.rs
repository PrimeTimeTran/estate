use crate::services::*;

#[derive(Default)]
pub struct ProblemServiceImpl<R> {
	repository: R,
}
impl<R> ProblemServiceImpl<R> {
	pub fn new(repository: R) -> Self {
		Self { repository }
	}
}
#[tonic::async_trait]
impl<R> ProblemService for ProblemServiceImpl<R>
where
	R: ProblemRepository + 'static,
{
	async fn list_problems(
		&self,
		request: Request<ListProblemsRequest>,
	) -> Result<Response<ListProblemsResponse>, Status> {
		let request = request.into_inner();
		let page = page_request(request.page)?;
		let result = self
			.repository
			.list(ProblemQuery {
				page: Some(page.page),
				page_size: Some(page.page_size),
				difficulty: request.difficulty,
			})
			.await
			.map_err(internal_error)?;
		Ok(Response::new(ListProblemsResponse {
			problems: result.items.clone(),
			page: Some(result.page_info()),
		}))
	}
	async fn create_problem(
		&self,
		request: Request<CreateProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		let request = request.into_inner();
		let problem = self
			.repository
			.create(CreateProblem {
				title: request.title,
				slug: request.slug,
			})
			.await
			.map_err(internal_error)?;
		Ok(Response::new(problem))
	}
	async fn update_problem(
		&self,
		request: Request<UpdateProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		let request = request.into_inner();
		let problem = UpdateProblem {
			title: request.title,
			slug: request.slug,
		};
		let id = problem_id(&request.id)?;
		let problem = self
			.repository
			.update(id, problem)
			.await
			.map_err(internal_error)?;
		Ok(Response::new(problem))
	}
	async fn delete_problem(
		&self,
		request: Request<DeleteProblemRequest>,
	) -> Result<Response<Empty>, Status> {
		let request = request.into_inner();
		let id = problem_id(&request.id)?;
		self.repository.delete(id).await.map_err(internal_error)?;
		Ok(Response::new(Empty {}))
	}
	async fn get_problem(
		&self,
		request: Request<GetProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		let request = request.into_inner();
		let id = problem_id(&request.id)?;
		let problem = self.repository.get(id).await.map_err(internal_error)?;
		Ok(Response::new(problem))
	}
	async fn sample_problem(
		&self,
		request: Request<SampleProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		let request = request.into_inner();

		let problem = self
			.repository
			.sample_problem(ProblemQuery {
				difficulty: request.difficulty,
				page_size: Some(1),
				page: Some(0),
			})
			.await
			.map_err(internal_error)?;

		Ok(Response::new(problem))
	}
}
fn problem_id(id: &str) -> Result<i64, Status> {
	id.parse()
		.map_err(|_| Status::invalid_argument("invalid problem id"))
}
