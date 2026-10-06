use super::*;

pub struct Context {}

// Identity & Persistence
#[async_trait::async_trait]
pub trait ArtifactGenerator: Send + Sync + Debug {
	async fn generate(&self, prompt: &str) -> Result<String>;
	async fn run_agent(&self, prompt: &str) -> Result<String>;
	async fn with_session(&mut self, session: &AiSession, prompt: String) -> Result<TaskResult>;
	fn clone_box(&self) -> Box<dyn ArtifactGenerator>;
}
// Steps to complete the pipeline
trait Pipeline {
	type Stage: Stage;
	fn id(&self) -> &PipelineId;
	fn state(&self) -> &PipelineState<Self::Stage>;
	fn name(&self) -> &'static str;
	fn description(&self) -> &'static str {
		""
	}
	fn stages(&self) -> &[Self::Stage];
}
#[async_trait::async_trait]
pub trait Runner {
	type Context;
	type Output;
	async fn run(&mut self, ctx: &mut Self::Context) -> Result<Self::Output>;
}
trait Stage {
	fn name(&self) -> &'static str;
	fn description(&self) -> &'static str {
		""
	}
	fn prepare(&self, _ctx: &mut Context) -> Result<()> {
		Ok(())
	}
	fn run(&self, ctx: &mut Context) -> Result<StageResult>;
	fn evaluate(&self, _ctx: &Context, _result: &StageResult) -> Result<Option<Evaluation>> {
		Ok(None)
	}
	fn transition(
		&self,
		_ctx: &Context,
		_result: &StageResult,
		_evaluation: Option<&Evaluation>,
	) -> Result<Transition> {
		Ok(Transition::Next)
	}
	fn cleanup(&self, _ctx: &mut Context) -> Result<()> {
		Ok(())
	}
}

impl Clone for Box<dyn ArtifactGenerator> {
	fn clone(&self) -> Self {
		self.as_ref().clone_box()
	}
}
pub trait TextModel {
	async fn generate(&self, prompt: &str) -> Result<String>;
}
