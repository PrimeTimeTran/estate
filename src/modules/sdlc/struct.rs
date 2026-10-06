use super::*;
use ratatui::Frame;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSession {
	pub id: Uuid,
	pub title: String,
	pub goal: String,
	pub stage: Stage,
	pub stages: Vec<StageRecord>,
	pub dir: PathBuf,
	pub time_created: DateTime<Utc>,
	pub time_updated: DateTime<Utc>,
	pub workspace: PathBuf,
}
pub struct AiView {
	pub runtime: PipelineRuntimeView,
	pub paused: bool,
	pub show_logs: bool,
	pub input: String,
	pub input_active: bool,
	pub events: Vec<Event>,
}
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
pub struct Context {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult;
pub struct CtxEvaluation {
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
	pub session: AiSession,
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
pub struct QACheck {
	pub confidence: f64,
	pub passed: bool,
	pub score: f64,
	pub stage: Stage,
	pub time_completed: DateTime<Utc>,
	pub time_started: DateTime<Utc>,
	pub time_total: chrono::Duration,
	pub evaluations: Vec<QAMetric>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAMetric {
	pub name: String,
	pub passed: bool,
	pub score: f64,
	pub confidence: f64,
	pub explanation: String,
}
#[derive(Debug)]
pub struct Execution {
	pub stage: Stage,
	pub attempt: Attempt,
	pub time_started: chrono::DateTime<Utc>,
	pub time_completed: chrono::DateTime<Utc>,
	pub result: StageResult,
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
	pub phase: Phase,
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
	pub phase: Phase,
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

#[derive(Clone, Debug)]
pub struct SprintPipeline {
	pub evaluator: Evaluator,
	pub generator: Box<dyn ArtifactGenerator>,
	pub session: AiSession,
	pub state_path: PathBuf,
	pub event_tx: broadcast::Sender<SdlcEvent>,
	pub stage_attempt: u32,
}
pub struct SprintRunner<'a> {
	pub pipeline: &'a mut SprintPipeline,
}

#[derive(Debug)]
pub struct StageError;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRecord {
	pub attempt: Attempt,
	pub description: Option<String>,
	pub stage: Stage,
	pub status: StageStatus,

	/// What actually performed the work.
	pub actor: StageActor,

	pub time_started: DateTime<Utc>,
	pub time_completed: Option<DateTime<Utc>>,
	
	/// Semantic evaluation of the resulting artifact/work.
	pub evaluation: Option<QACheck>,
}

pub struct TerminalGuard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
	pub passed: bool,
	// pub score: u32,
	pub checks: Vec<CheckResult>,
	pub evaluations: Vec<QAMetric>,
}

#[derive(Debug, Clone)]
pub struct WSChanges {
	pub git_status: String,
}
pub struct WSSnapshot {
	pub git_status: String,
}

