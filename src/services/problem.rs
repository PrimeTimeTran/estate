use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::{
	leetcode::{problem_service_server::ProblemService, *},
	prelude::*,
	proto::leetcode::{
		CreateProblemRequest, DeleteProblemRequest, Empty, GetProblemRequest, ListProblemsRequest,
		ListProblemsResponse, Problem, UpdateProblemRequest,
	},
	repo::{
		Page,
		problem::{CreateProblem, ProblemQuery, ProblemRepository, UpdateProblem},
	},
};

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
impl<R> crate::proto::leetcode::problem_service_server::ProblemService for ProblemServiceImpl<R>
where
	R: ProblemRepository + 'static,
{
	async fn list_problems(
		&self,
		request: Request<ListProblemsRequest>,
	) -> Result<Response<ListProblemsResponse>, Status> {
		let request = request.into_inner();
		let page = request.page.unwrap();
		let query = ProblemQuery {
			page: Some(page.page),
			page_size: Some(page.page_size),
			difficulty: request.difficulty,
		};
		let result = self.repository.list(query).await.map_err(internal_error)?;
		Ok(Response::new(ListProblemsResponse {
			problems: result.items,
			page: Some(crate::proto::leetcode::PageInfo {
				page: result.page as i32,
				page_size: result.page_size as i32,
				total: result.total as i64,
			}),
		}))
	}

	async fn create_problem(
		&self,
		request: Request<CreateProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		let request = request.into_inner();

		let problem = CreateProblem {
			title: request.title,
			slug: request.slug,
		};

		let problem = self
			.repository
			.create(problem)
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
		let id = request
			.id
			.parse::<i64>()
			.map_err(|_| Status::invalid_argument("invalid problem id"))?;

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

		let id = request
			.id
			.parse::<i64>()
			.map_err(|_| Status::invalid_argument("invalid problem id"))?;

		self.repository.delete(id).await.map_err(internal_error)?;

		Ok(Response::new(Empty {}))
	}

	async fn get_problem(
		&self,
		request: Request<GetProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		let request = request.into_inner();

		let id = request
			.id
			.parse::<i64>()
			.map_err(|_| Status::invalid_argument("invalid problem id"))?;

		let problem = self.repository.get(id).await.map_err(internal_error)?;

		Ok(Response::new(problem))
	}
}

fn internal_error(error: anyhow::Error) -> Status {
	tracing::error!("{error:#}");
	Status::internal(error.to_string())
}
