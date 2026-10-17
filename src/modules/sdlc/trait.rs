use super::*;

impl Clone for Box<dyn GeneratorTrait> {
	fn clone(&self) -> Self {
		self.as_ref().clone_box()
	}
}

#[async_trait::async_trait]
pub trait GeneratorTrait: Send + Sync + Debug {
	fn clone_box(&self) -> Box<dyn GeneratorTrait>;

	async fn generate(&self, prompt: &str) -> Result<String>;
	async fn run_agent(&self, prompt: &str) -> Result<String>;
	async fn with_session(&mut self, session: &AiSession, prompt: String) -> Result<TaskResult>;
}

// Steps to complete the pipeline
pub trait PipelineTrait {
	type Stage: StageTrait;

	fn id(&self) -> &PipelineId;
	fn name(&self) -> &'static str;
	fn description(&self) -> &'static str {
		""
	}
	fn stages(&self) -> &[Self::Stage];
	fn stage(&self, name: &str) -> Option<&Self::Stage> {
		self.stages().iter().find(|stage| stage.name() == name)
	}
	fn first_stage(&self) -> Option<&Self::Stage> {
		self.stages().first()
	}
	fn next_stage(&self, current: &str) -> Option<&Self::Stage> {
		let stages = self.stages();
		let index = stages.iter().position(|stage| stage.name() == current)?;
		stages.get(index + 1)
	}
	fn previous_stage(&self, current: &str) -> Option<&Self::Stage> {
		let stages = self.stages();
		let index = stages.iter().position(|stage| stage.name() == current)?;
		index.checked_sub(1).and_then(|index| stages.get(index))
	}
	fn validate(&self) -> Result<()> {
		let stages = self.stages();
		anyhow::ensure!(!stages.is_empty(), "Pipeline has no stages");
		for (index, stage) in stages.iter().enumerate() {
			anyhow::ensure!(
				!stages[..index]
					.iter()
					.any(|previous| previous.name() == stage.name()),
				"Duplicate stage name: {}",
				stage.name(),
			);
		}
		Ok(())
	}
}
#[async_trait::async_trait]
pub trait RunnerTrait {
	type Context;
	type Output;
	async fn run(&mut self, ctx: &mut Self::Context) -> Result<Self::Output>;
}
pub trait StageTrait {
	fn checkpoint(&self, ctx: &mut Context) -> Result<()> {
		// ctx.session.save()?;
		// ctx.runtime.log_dir()?;
		// ctx.runtime.write_artifact()?;
		Ok(())
	}
	fn cleanup(&self, _ctx: &mut Context) -> Result<()> {
		Ok(())
	}
	fn description(&self) -> &'static str {
		""
	}
	fn evaluate(&self, _ctx: &Context, _result: &RunResult) -> Result<Option<Evaluation>> {
		Ok(None)
	}
	fn name(&self) -> &'static str;
	fn prepare(&self, _ctx: &mut Context) -> Result<()> {
		Ok(())
	}
	fn resume(&self, _ctx: &mut Context) -> Result<()> {
		Ok(())
	}
	fn run(&self, ctx: &mut Context) -> Result<RunResult>;
	fn transition(
		&self,
		_ctx: &Context,
		_result: &RunResult,
		_evaluation: Option<&Evaluation>,
	) -> Result<Transition> {
		Ok(Transition::Next)
	}
	fn validate(&self, _ctx: &mut Context) -> Result<()> {
		Ok(())
	}

	fn on_failure(&self, _ctx: &mut Context) -> Result<()> {
		Ok(())
	}
}

pub trait TextModel {
	async fn generate(&self, prompt: &str) -> Result<String>;
}

pub struct Context {
	pub stage: Stage,
	pub phase: Phase,
	pub attempt: u32,

	pub session: AiSession,
	pub log_dir: PathBuf,
	// Runtime-provided services.
	// pub storage: Arc<dyn ArtifactStore>,
	// pub logger: Arc<dyn Logger>,
}
