use std::sync::LazyLock;

use super::*;

pub const TODO: &'static str = r#"
	- Question prompt
	- Add progressive disclosure
"#;

pub static SDLC_LOG_DIR: LazyLock<PathBuf> =
	LazyLock::new(|| PathBuf::from("/Users/future/kb/project/crates/estate/log"));

pub static DEFAULT_NAUL_BAR: f64 = 0.40;
pub static DEFAULT_MODEL: &str = "qwen3:8b";
pub static SDLC_GOAL: &str =
	include_str!("/Users/future/kb/project/crates/estate/ai/template/user.goal.md");

pub static SYS_PROMPT_INTENT: &str =
	"/Users/future/kb/project/crates/estate/log/sys-prompt-intent.md";
pub static SYS_PROMPT_SPEC: &str = "/Users/future/kb/project/crates/estate/log/sys-prompt-spec.md";
pub static SYS_PROMPT_PLAN: &str = "/Users/future/kb/project/crates/estate/log/sys-prompt-plan.md";

pub static OLLAMA_REQUEST: &str = "/Users/future/kb/project/crates/estate/log/ollama-request.json";
pub static OLLAMA_RESPONSE: &str =
	"/Users/future/kb/project/crates/estate/log/ollama-response.json";

pub const LOG_EVENT_NAME: &str = "events.jsonl";
pub const INTENT_PROMPT: &str =
	include_str!("/Users/future/kb/project/crates/estate/ai/template/INITIAL_PROMPT.md");
pub const DEMO_COMPLETE_DELAY: Duration = Duration::from_secs(1);
pub const DEMO_EVALUATION_TIME: Duration = Duration::from_secs(1);
pub const DEMO_EXECUTION_TIME: Duration = Duration::from_secs(1);
pub const DEMO_RETRY_DELAY: Duration = Duration::from_secs(1);
pub const FMT_HUMAN_READABLE: &'static str = "%B %-d, %Y at %-I:%M:%S %p UTC";
pub const MAX_STAGE_ATTEMPTS: u32 = 100;
pub const STATUS_INTERVAL: Duration = Duration::from_secs(30);
