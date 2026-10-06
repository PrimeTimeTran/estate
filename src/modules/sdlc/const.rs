use super::*;

pub const TODO: &'static str = r#"
	- Question prompt
	- Add progressive disclosure
"#;

pub static DEFAULT_MODEL: &str = "qwen3:8b";
pub static SDLC_GOAL: &str = include_str!("/Users/future/kb/project/crates/estate/ai/template/user.goal.md");
pub static SDLC_ARTIFACT_INTENT: &str = "/Users/future/kb/project/crates/estate/log/intent-prompt.md";
pub static OLLAMA_REQUEST: &str = "/Users/future/kb/project/crates/estate/log/ollama-request.json";
pub static OLLAMA_RESPONSE: &str =
	"/Users/future/kb/project/crates/estate/log/ollama-response.json";
pub const LOG_EVENT_NAME: &str = "events.jsonl";
pub const INTENT_PROMPT: &str = include_str!("/Users/future/kb/project/crates/estate/ai/template/INITIAL_PROMPT.md");
pub const DEMO_COMPLETE_DELAY: Duration = Duration::from_secs(1);
pub const DEMO_EVALUATION_TIME: Duration = Duration::from_secs(1);
pub const DEMO_EXECUTION_TIME: Duration = Duration::from_secs(1);
pub const DEMO_RETRY_DELAY: Duration = Duration::from_secs(1);
pub const FMT_HUMAN_READABLE: &'static str = "%B %-d, %Y at %-I:%M:%S %p UTC";
pub const MAX_STAGE_ATTEMPTS: u32 = 100;
pub const STATUS_INTERVAL: Duration = Duration::from_secs(30);
