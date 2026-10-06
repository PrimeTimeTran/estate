use super::*;

#[derive(Debug)]
pub enum ExecutionResult {
	Completed(TaskResult),
	Failed(StageError),
}

pub enum InputMode {
	Normal,
	Human { buffer: String },
	AwaitingHuman { prompt: String },
}
#[derive(Debug, Clone)]
pub enum Intervention {
	Human(String),
	Retry,
	ProvideContext(String),
	Reviewed,
	Abort,
	Revise,
	Revision { evaluation: StageEvaluation },
}

#[derive(Debug, Clone, Copy)]
pub enum GenerationProvider {
	Local,
	Api,
}

pub type PipelineId = uuid::Uuid;

#[derive(Debug)]
pub enum RunControl {
	Continue,
	Exit,
	RetryStage,
}
pub enum RunResult {
	Completed,
	Failed,
	Cancelled,
}
pub enum RunState {
	Idle,
	Running,
	Paused,
	AwaitingInput,
	Completed,
	Failed,
	Cancelled,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SdlcEvent {
	Activity {
		stage: Stage,
		attempt: Attempt,
		message: String,
	},
	Completed,
	Failed {
		stage: Option<Stage>,
		error: String,
	},
	InterventionRequired {
		stage: Stage,
		attempt: Attempt,
		reason: String,
	},
	InterventionResolved {
		stage: Stage,
		action: String,
	},
	Evaluated {
		stage: Stage,
		score: f64,
		confidence: f64,
		passed: bool,
	},
	EvaluationStarted {
		stage: Stage,
	},
	EvaluationFailed {
		stage: Stage,
		error: String,
	},
	ExecutionComplete {
		stage: Stage,
	},
	ExecutionFailed {
		stage: Stage,
		attempt: Attempt,
		error: String,
	},
	Exited {
		reason: String,
	},
	PhaseChanged {
		phase: SdlcPhase,
	},
	RunStarted,
	StageRetrying {
		stage: Stage,
		number: u32,
	},
	StageStarted {
		stage: Stage,
		attempt: Attempt,
	},
	StageTransitioned {
		from: Stage,
		to: Stage,
	},
	HumanInput {
		stage: Stage,
		input: String,
	},
}
#[derive(Debug)]
pub enum SdlcInput {
	Abort,
	ProvideContext(String),
	Retry,
	Reviewed,
	Revision { evaluation: StageEvaluation },
	Human(String),
}
#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub enum SdlcPhase {
	Starting,
	Executing,
	Evaluating,
	Retrying,
	AwaitingHuman,
	Completed,
	Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
	Intent,
	Spec,
	Plan,
	Test,
	Build,
	Verify,
	Deploy,
	Maintain,
	Complete,
	Finalize,
}

#[derive(Debug, Clone, Copy)]
pub enum StageAction {
	Continue,
	Retry,
	Intervene,
	Abort,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StageActor {
	Human,
	Sdlc,
	Evaluator,
	Agent,
	System,
}
#[derive(Debug)]
pub enum StageDecision {
	Continue,
	Retry,
	Revise,
	AwaitHuman,
	Fail,
	Complete,
	Exit,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageInput {
	Initial,
	Revision { evaluation: StageEvaluation },
}

#[derive(Debug)]
pub enum StageOutcome {
	Complete {
		execution: StageExecution,
		evaluation: StageEvaluation,
	},
	NeedsRevision {
		execution: StageExecution,
		evaluation: StageEvaluation,
	},
	ExecutionFailed {
		stage: Stage,
		attempt: Attempt,
		error: anyhow::Error,
	},
	EvaluationFailed {
		execution: StageExecution,
		error: anyhow::Error,
	},
}
pub enum StageOutcomeEvaluation {
	Passed(Evaluation),
	FailedQuality(Evaluation),
	FailedRuntime(Error),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageResult {
	Intent,
	Spec,
	Plan,
	Build,
	Verification(Verification),
	Complete,
	Finalize,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StageStatus {
	Running,
	Completed,
	Failed,
	NeedsRevision,
	EvaluationFailed,
	InterventionNeeded,
}

pub enum StageTransition {
	Next,
	Repeat,
	Goto(Stage),
	Complete,
	Fail,
	AwaitHuman,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
	Boot,
	Init,
	Intent,
	Spec,
	Plan,
	Build,
	Verify,
	Deploy,
	Maintain,
	Complete,
}
