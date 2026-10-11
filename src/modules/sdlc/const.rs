use std::sync::LazyLock;

use super::*;

pub const TODO: &'static str = r#"
	- Question prompt
	- Add progressive disclosure
"#;

pub const SDLC_LOG_DIR: &str = "/Users/future/kb/project/crates/estate/log";
pub const SDLC_LOG_FILE_NAME: &str = "events.jsonl";

pub static DEFAULT_LOOP_TRIES: u32 = 3;
pub static DEFAULT_NAUL_BAR: f64 = 0.40;
pub static DEFAULT_MODEL: &str = "qwen3:8b";

pub const SDLC_TEMPLATE_INTENT: &str = include_str!("input/intent.md");
pub const SDLC_TEMPLATE_SPEC: &str = include_str!("input/spec.md");
pub const SDLC_TEMPLATE_PLAN: &str = include_str!("input/plan.md");

pub const SDLC_PROMPT_INTENT: &str = include_str!("input/prompt_intent.md");
pub const SDLC_PROMPT_SPEC: &str = include_str!("input/prompt_spec.md");
pub const SDLC_PROMPT_PLAN: &str = include_str!("input/prompt_plan.md");

pub const SDLC_GOAL: &str = include_str!("input/user.goal.md");

pub const PROMPT_BUILD_ORIENT: &str = include_str!("input/PROMPT_BUILD_ORIENT.md");
pub const PROMPT_BUILD_EXECUTE: &str = include_str!("input/PROMPT_BUILD_EXECUTE.md");
pub const PROMPT_BUILD_VERIFY: &str = include_str!("input/PROMPT_BUILD_VERIFY.md");

pub static SYS_PROMPT_INTENT: &str =
	"/Users/future/kb/project/crates/estate/log/SYS_PROMPT_INTENT.md";
pub static SYS_PROMPT_SPEC: &str = "/Users/future/kb/project/crates/estate/log/SYS_PROMPT_SPEC.md";
pub static SYS_PROMPT_PLAN: &str = "/Users/future/kb/project/crates/estate/log/SYS_PROMPT_PLAN.md";

pub static OLLAMA_REQUEST: &str = "/Users/future/kb/project/crates/estate/log/ollama-request.json";
pub static OLLAMA_RESPONSE: &str =
	"/Users/future/kb/project/crates/estate/log/ollama-response.json";

pub const INTENT_PROMPT: &str =
	include_str!("/Users/future/kb/project/crates/estate/ai/template/INITIAL_PROMPT.md");
pub const DEMO_COMPLETE_DELAY: Duration = Duration::from_secs(1);
pub const DEMO_EVALUATION_TIME: Duration = Duration::from_secs(1);
pub const DEMO_EXECUTION_TIME: Duration = Duration::from_secs(1);
pub const DEMO_RETRY_DELAY: Duration = Duration::from_secs(1);
pub const FMT_HUMAN_READABLE: &'static str = "%B %-d, %Y at %-I:%M:%S %p UTC";
pub const MAX_STAGE_ATTEMPTS: u32 = 100;
pub const STATUS_INTERVAL: Duration = Duration::from_secs(30);
