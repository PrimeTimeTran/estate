use std::sync::LazyLock;

use super::*;

pub const TODO: &'static str = r#"
	- Question prompt
	- Add progressive disclosure
"#;

pub const SDLC_LOG_DIR: &str = "/Users/future/kb/project/crates/estate/log";
pub const SDLC_LOG_FILE_NAME: &str = "events.jsonl";

pub static DEFAULT_NAUL_BAR: f64 = 0.40;
pub static DEFAULT_MODEL: &str = "qwen3:8b";

pub static SDLC_TEMPLATE_INTENT: &str =
	"/Users/future/kb/project/crates/estate/src/modules/sdlc/input/intent.md";
pub static SDLC_TEMPLATE_SPEC: &str =
	"/Users/future/kb/project/crates/estate/src/modules/sdlc/input/spec.md";
pub static SDLC_TEMPLATE_PLAN: &str =
	"/Users/future/kb/project/crates/estate/src/modules/sdlc/input/plan.md";

pub static SDLC_PROMPT_INTENT: &str =
	"/Users/future/kb/project/crates/estate/src/modules/sdlc/input/prompt_intent.md";
pub static SDLC_PROMPT_SPEC: &str =
	"/Users/future/kb/project/crates/estate/src/modules/sdlc/input/prompt_spec.md";
pub static SDLC_PROMPT_PLAN: &str =
	"/Users/future/kb/project/crates/estate/src/modules/sdlc/input/prompt_plan.md";

pub static SDLC_GOAL: &str =
	"/Users/future/kb/project/crates/estate/src/modules/sdlc/input/user.goal.md";

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

pub const PROMPT_BUILD_ORIENT: &str = r#"You are in the ORIENT phase of the SDLC Build stage.

Your objective is to understand the implementation requirements and
establish the current state of the workspace before making changes.

## Required actions

1. Read and understand plan.md.
2. Read intent.md and spec.md for the original goal and requirements.
3. Inspect the repository structure and relevant existing files.
4. Inspect Git status, including modified and untracked files.
5. For every expected file in the Plan, determine whether it exists
   and whether its required behavior is already implemented.
6. Identify missing work, incomplete work, existing work, and blockers.

## Rules

- Do not modify, create, or delete implementation files in this phase.
- A clean Git status does not mean the task is complete.
- Existing files do not automatically satisfy their requirements.
- Do not assume planned changes are necessary if the requirements
  are already satisfied; gather evidence first.
- Preserve unrelated changes.
- Do not invent requirements beyond the Intent, Specification, and Plan.

## Required output

Return an orientation report containing:

1. Task objective.
2. Required file changes and their purposes.
3. Current state of each required file.
4. Outstanding implementation requirements.
5. Verification requirements.
6. Relevant risks, ambiguities, or blockers.

Do not begin implementation. Finish after the report is complete.
"#;


pub const PROMPT_BUILD_EXECUTE: &str = r#"You are in the EXECUTE phase of the SDLC Build stage.

Your objective is to implement the outstanding requirements identified
during orientation, following the Plan and respecting the Specification.

## Required actions

1. Review the orientation report and previous action history.
2. Confirm the current workspace state before making changes.
3. Implement the outstanding requirements in the Plan.
4. Create, modify, or delete only files justified by the requirements.
5. Follow existing project conventions and use the appropriate
   language, tools, dependencies, and project structure.
6. After meaningful changes, inspect the results and run relevant checks.
7. Record actual command results and any remaining failures.

## Rules

- Do not repeat completed work without a specific reason.
- Do not overwrite or discard unrelated existing changes.
- Do not silently omit planned requirements.
- Do not claim a command succeeded unless its result confirms success.
- If a requirement is ambiguous or blocked, report the specific issue.
- Do not claim implementation is verified merely because files exist.
- Keep changes incremental and consistent with the Plan.

## Required output

Return an implementation report containing:

1. Requirements addressed.
2. Files created, modified, or deleted.
3. Important implementation decisions.
4. Commands executed and their actual outcomes.
5. Outstanding work, errors, or blockers.

Finish when the planned implementation is complete or further progress
is blocked. Verification will be assessed in the next phase.
"#;


pub const PROMPT_BUILD_VERIFY: &str = r#"You are in the VERIFY phase of the SDLC Build stage.

Your objective is to independently verify that the implementation
satisfies the Plan, Specification, and task acceptance criteria.

## Required actions

1. Re-read the relevant requirements in plan.md and spec.md.
2. Inspect the actual final workspace and implementation changes.
3. Confirm that every required file exists and has the expected behavior.
4. Run the appropriate unit tests, integration tests, builds, or
   CLI checks required by the Plan.
5. Inspect command exit codes, stdout, and stderr for failures.
6. Compare the implementation against each acceptance criterion.
7. Identify regressions, missing requirements, and unverified behavior.

## Rules

- Treat implementation reports as claims that must be checked.
- Do not treat a clean Git status as proof of correctness.
- Do not treat file existence as proof of correct behavior.
- Do not claim tests passed if they were not run successfully.
- Distinguish passed, failed, blocked, and not-run checks.
- Do not conceal failures or change acceptance criteria to pass.
- If defects are found, report the specific defect and required correction.
- Do not declare success while required acceptance criteria remain unmet.

## Required output

Return a verification report containing:

1. Each acceptance criterion and its status.
2. Files and behaviors inspected.
3. Verification commands and actual results.
4. Failures, risks, and unverified requirements.
5. Overall verdict: PASS, FAIL, or BLOCKED.

PASS means the required criteria have sufficient verification evidence.
FAIL means one or more criteria are not satisfied.
BLOCKED means verification cannot be completed because a dependency
or required decision is unavailable.
"#;
