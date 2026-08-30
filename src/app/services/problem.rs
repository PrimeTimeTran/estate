use tonic::{Request, Response, Status};

use crate::{
	leetcode::{problem_service_server::ProblemService, *},
	prelude::*,
};

#[derive(Default)]
pub struct ProblemServiceImpl;

#[tonic::async_trait]
impl ProblemService for ProblemServiceImpl {
	async fn list_problems(
		&self,
		_request: Request<ListProblemsRequest>,
	) -> Result<Response<ListProblemsResponse>, Status> {
		Ok(Response::new(ListProblemsResponse {
			page: Some(PageInfo {
				page: 0,
				page_size: 0,
				total: 0,
			}),
			problems: vec![],
			// Add any other required fields from your proto here.
		}))
	}
	async fn create_problem(
		&self,
		_request: Request<CreateProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		todo!("create")
	}
	async fn update_problem(
		&self,
		_request: Request<UpdateProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		todo!("update")
	}
	async fn delete_problem(
		&self,
		_request: Request<DeleteProblemRequest>,
	) -> Result<Response<Empty>, Status> {
		todo!("delete_problem")
	}
	async fn get_problem(
		&self,
		_request: Request<GetProblemRequest>,
	) -> Result<Response<Problem>, Status> {
		todo!("get_problem")
	}
}

use crate::proto::leetcode::{
	CreateProblemRequest, DeleteProblemRequest, Empty, GetProblemRequest, ListProblemsRequest,
	ListProblemsResponse, PageInfo, Problem, UpdateProblemRequest,
};
