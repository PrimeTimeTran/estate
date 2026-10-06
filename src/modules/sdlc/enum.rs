use super::*;

#[derive(Debug)]
pub enum Decision {
	Continue,
	Retry,
	Revise,
	AwaitHuman,
	Fail,
	Complete,
	Exit,
}

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
	Revision { evaluation: QACheck },
}

#[derive(Debug, Clone, Copy)]
pub enum GenerationProvider {
	Local,
	Api,
}

#[derive(Debug, Clone, Deserialize, Serialize, Copy, PartialEq)]
pub enum Phase {
	Starting,
	Executing,
	Evaluating,
	Retrying,
	AwaitingHuman,
	Completed,
	Failed,
}
pub type PipelineId = uuid::Uuid;

#[derive(Debug)]
pub enum RunControl {
	Continue,
	Exit,
	RetryStage,
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
		phase: Phase,
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
	Revision { evaluation: QACheck },
	Human(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
	Intent,
	Spec,
	Plan,
	Test,
	Build,
	QA,
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
	Runner,
	Evaluator,
	Agent,
	System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StageInput {
	Initial,
	Revision { evaluation: QACheck },
}

#[derive(Debug)]
pub enum Outcome {
	Complete {
		execution: Execution,
		evaluation: QACheck,
	},
	NeedsRevision {
		execution: Execution,
		evaluation: QACheck,
	},
	ExecutionFailed {
		stage: Stage,
		attempt: Attempt,
		error: anyhow::Error,
	},
	EvaluationFailed {
		execution: Execution,
		error: anyhow::Error,
	},
}
pub enum StageOutcomeEvaluation {
	Passed(Evaluation),
	FailedQuality(Evaluation),
	FailedRuntime(Error),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunResult {
	Intent,
	Spec,
	Plan,
	Build,
	Verification(Verification),
	Complete,
	Finalize,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Status {
	Running,
	Completed,
	Failed,
	NeedsRevision,
	EvaluationFailed,
	InterventionNeeded,
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

pub enum Transition {
	Next,
	Repeat,
	Goto(Stage),
	Complete,
	Fail,
	AwaitHuman,
}
