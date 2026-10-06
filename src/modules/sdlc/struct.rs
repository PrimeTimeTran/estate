use super::*;
use ratatui::Frame;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Attempt {
	pub stage: Stage,
	pub number: u32,
	pub max: u32,
}
#[derive(Clone, Debug)]
pub struct ApiGenerator {
	// whatever API client you decide to use
}
pub struct BuildResult {
	pub changed_files: Vec<PathBuf>,
	pub created_files: Vec<PathBuf>,
	pub modified_files: Vec<PathBuf>,
	pub deleted_files: Vec<PathBuf>,
	pub commands: Vec<CommandResult>,
	pub agent_summary: Option<String>,
}
pub struct Check {
	pub name: String,
	pub command: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
	pub name: String,
	pub passed: bool,
	pub output: Option<String>,
}
pub struct CommandResult {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult;

pub struct EvaluationContext {
	pub stage: Stage,
	pub intent: Option<String>,
	pub spec: Option<String>,
	pub plan: Option<String>,
	pub tests: Option<String>,
	pub progress: Option<String>,
	pub verification: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Evaluator {
	pub jev: TypeSafeClient,
	pub session: SdlcSession,
}
#[derive(Debug, Clone)]
pub struct Evaluation {
	pub score: f64,
	pub confidence: f64,
	pub meets_bar: bool,
	pub feedback: String,
	pub criteria: Vec<CriterionResult>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
	pub name: String,
	pub passed: bool,
	pub score: f64,
	pub confidence: f64,
	pub explanation: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
	pub score: f64,
	pub confidence: f64,
	pub meets_bar: bool,
	pub feedback: String,
}

#[derive(Clone, Debug)]
pub struct LocalGenerator {
	pub runtime: AgentRuntime,
	pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
	pub model: String,
	pub response: String,
	pub done: bool,
	pub done_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PipelineRuntime {
	pub activity: Vec<String>,
	pub pipeline: SprintPipeline,
	pub attempt: u32,
	pub stage: e::Stage,
	pub time_started: Instant,
	pub stage_time_started: Instant,
	pub phase: SdlcPhase,
	pub score: Option<f64>,
	pub confidence: Option<f64>,
	pub passed: Option<bool>,
	pub message: Option<String>,
	pub error: Option<String>,
	pub total_tokens: u64,
	pub total_agent_calls: u32,
	pub history: Vec<SdlcEvent>,
	pub events: Vec<SdlcEvent>,
}
#[derive(Debug, Clone)]
pub struct PipelineRuntimeView {
	pub activity: Vec<String>,
	pub attempt: u32,
	pub stage: e::Stage,
	pub time_started: Instant,
	pub stage_time_started: Instant,
	pub phase: SdlcPhase,
	pub score: Option<f64>,
	pub confidence: Option<f64>,
	pub passed: Option<bool>,
	pub message: Option<String>,
	pub error: Option<String>,
	pub total_tokens: u64,
	pub total_agent_calls: u32,
	pub history: Vec<SdlcEvent>,
	pub events: Vec<SdlcEvent>,
}
pub struct PipelineState<S> {
	stage: S,
	attempt: u32,
	status: PipelineStatus,
}
pub struct PipelineStatus;

pub struct RetryPolicy {
	pub max_attempts: u32,
	pub retry_execution: bool,
	pub retry_evaluation: bool,
	pub retry_quality: bool,
	pub allow_human_intervention: bool,
}

#[derive(Clone, Debug)]
pub struct SprintPipeline {
	pub evaluator: Evaluator,
	pub generator: Box<dyn ArtifactGenerator>,
	pub session: Option<SdlcSession>,
	pub state_path: PathBuf,
	// pub event_tx: broadcast::Sender<SdlcEvent>,
	pub event_tx: broadcast::Sender<SdlcEvent>,
	pub stage_attempt: u32,
}
pub struct SprintRunner<'a> {
	pub pipeline: &'a mut SprintPipeline,
}
#[derive(Debug)]
pub struct StageError;
#[derive(Debug)]
pub struct StageExecution {
	pub stage: e::Stage,
	pub attempt: Attempt,
	pub time_started: chrono::DateTime<Utc>,
	pub time_completed: chrono::DateTime<Utc>,
	pub result: StageResult,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageEvaluation {
	pub stage: e::Stage,
	pub actor: StageActor,
	pub time_started: DateTime<Utc>,
	pub time_completed: DateTime<Utc>,
	pub time_total: chrono::Duration,
	pub score: f64,
	pub confidence: f64,
	pub passed: bool,

	pub evaluations: Vec<EvaluationResult>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRecord {
	pub attempt: Attempt,
	pub stage: e::Stage,
	pub status: StageStatus,
	pub description: Option<String>,

	/// What actually performed the work.
	pub actor: StageActor,

	pub time_started: DateTime<Utc>,
	pub time_completed: Option<DateTime<Utc>>,
	// time_started: time_readable(time_started),
	// time_completed: time_readable(time_completed),
	// time_total: duration_readable(time_completed - time_started),
	/// Semantic evaluation of the resulting artifact/work.
	pub evaluation: Option<StageEvaluation>,
}
pub struct StageRevision {
	pub previous_artifact: String,
	pub feedback: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdlcSession {
	pub id: Uuid,
	pub title: String,
	pub goal: String,
	pub stage: e::Stage,
	pub stages: Vec<StageRecord>,
	pub dir: PathBuf,
	pub time_created: DateTime<Utc>,
	pub time_updated: DateTime<Utc>,
	pub workspace: PathBuf,
}
pub struct SdlcView {
	pub runtime: PipelineRuntimeView,
	pub paused: bool,
	pub show_logs: bool,
	pub input: String,
	pub input_active: bool,
	pub events: Vec<Event>,
}

pub struct TerminalGuard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
	pub passed: bool,
	// pub score: u32,
	pub checks: Vec<CheckResult>,
	pub evaluations: Vec<EvaluationResult>,
}

pub struct WorkspaceSnapshot {
	pub git_status: String,
}
#[derive(Debug, Clone)]
pub struct WorkspaceChanges {
	pub git_status: String,
}
