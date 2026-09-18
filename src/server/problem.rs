use crate::{
	ProblemService as ProblemServiceTrait,
	model::{ProtoProblem, common::Difficulty},
	prelude::*,
	server::*,
};

fn problem_id(id: &str) -> Result<i64, Status> {
	id.parse()
		.map_err(|_| Status::invalid_argument("invalid problem id"))
}

#[tonic::async_trait]
impl<R> ProblemServiceTrait for ProblemService<R>
where
	R: ProblemRepository + 'static,
{
	async fn create_problem(
		&self,
		request: Request<CreateProblemRequest>,
	) -> Result<Response<ProtoProblem>, Status> {
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
	async fn list_problems(
		&self,
		request: Request<ListProblemsRequest>,
	) -> Result<Response<ListProblemsResponse>, Status> {
		let request = request.into_inner();
		let page = page_request(request.page)?;

		let difficulty = request
			.difficulty
			.map(Difficulty::try_from)
			.transpose()
			.map_err(internal_error)?;

		let result = self
			.repository
			.list(ProblemQuery {
				page: Some(page.page),
				page_size: Some(page.page_size),
				difficulty,
			})
			.await
			.map_err(internal_error)?;

		Ok(Response::new(ListProblemsResponse {
			problems: result.items.clone(),
			page: Some(result.page_info()),
		}))
	}
	async fn update_problem(
		&self,
		request: Request<UpdateProblemRequest>,
	) -> Result<Response<ProtoProblem>, Status> {
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
	) -> Result<Response<ProtoProblem>, Status> {
		let request = request.into_inner();
		let id = problem_id(&request.id)?;
		let problem = self.repository.get(id).await.map_err(internal_error)?;
		Ok(Response::new(problem))
	}
	async fn sample_problem(
		&self,
		request: Request<SampleProblemRequest>,
	) -> Result<Response<ProtoProblem>, Status> {
		let request = request.into_inner();
		let difficulty = request
			.difficulty
			.map(Difficulty::try_from)
			.transpose()
			.map_err(internal_error)?;

		let problem = self
			.repository
			.sample_problem(ProblemQuery {
				difficulty,
				page_size: Some(1),
				page: Some(0),
			})
			.await
			.map_err(internal_error)?;

		Ok(Response::new(problem))
	}
}

impl<R> ProblemService<R> {
	pub fn new(repository: R) -> Self {
		Self { repository }
	}
}

pub struct CreateProblem {
	pub title: String,
	pub slug: String,
}
pub struct ProblemQuery {
	pub page: Option<i32>,
	pub page_size: Option<i32>,
	pub difficulty: Option<Difficulty>,
}

impl TryFrom<ProblemQuery> for crate::proto::types::ListProblemsRequest {
	type Error = anyhow::Error;

	fn try_from(query: ProblemQuery) -> Result<Self, Self::Error> {
		let page = query.page.map(|page| crate::proto::types::PageRequest {
			page,
			page_size: query.page_size.unwrap_or(20),
		});

		Ok(Self {
			page,
			difficulty: query.difficulty.map(i32::from),
			tags: Vec::new(),
			search: String::new(),
			published_only: Some(false),
		})
	}
}
#[derive(Default)]
pub struct ProblemService<R> {
	repository: R,
}
pub struct UpdateProblem {
	pub title: Option<String>,
	pub slug: Option<String>,
}
