use super::*;

pub const TODO: &'static str = r#"
	- Question prompt
	- Add progressive disclosure
"#;

pub const INTENT_PROMPT: &str = include_str!("../../../ai/template/INITIAL_PROMPT.md");
pub const PROMPT_FROM_USER: &str = include_str!("../../../ai/template/user.goal.md");
pub const DEMO_COMPLETE_DELAY: Duration = Duration::from_secs(1);
pub const DEMO_EVALUATION_TIME: Duration = Duration::from_secs(1);
pub const DEMO_EXECUTION_TIME: Duration = Duration::from_secs(1);
pub const DEMO_RETRY_DELAY: Duration = Duration::from_secs(1);
pub const FMT_HUMAN_READABLE: &'static str = "%B %-d, %Y at %-I:%M:%S %p UTC";
pub const MAX_STAGE_ATTEMPTS: u32 = 1;
pub const STATUS_INTERVAL: Duration = Duration::from_secs(30);

