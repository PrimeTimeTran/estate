use super::*;

pub fn for_intent(user_request: &str) -> String {
	c::INTENT_PROMPT.replace("{{SDLC_GOAL}}", user_request)
}

pub fn gen_intent(goal: &str) -> Result<String> {
	if goal.trim().is_empty() {
		return Err(anyhow!("cannot generate Intent prompt from empty goal"));
	}

	let prompt_template = std::fs::read_to_string(SDLC_PROMPT_INTENT)?;
	let template = std::fs::read_to_string(SDLC_TEMPLATE_INTENT)?;

	let prompt = prompt_template
		.replace("{template}", &template)
		.replace("{goal}", goal);

	Ok(prompt)
}

pub fn gen_spec(intent: &str) -> Result<String> {
	if intent.trim().is_empty() {
		return Err(anyhow!("cannot generate Spec prompt from empty Intent"));
	}

	let prompt_template = std::fs::read_to_string(SDLC_PROMPT_SPEC)?;
	let template = std::fs::read_to_string(SDLC_TEMPLATE_SPEC)?;

	let prompt = prompt_template
		.replace("{template}", &template)
		.replace("{intent}", intent);

	Ok(prompt)
}

pub fn gen_plan(intent: &str, spec: &str) -> Result<String> {
	if intent.trim().is_empty() {
		return Err(anyhow!("Intent artifact is empty"));
	}
	if spec.trim().is_empty() {
		return Err(anyhow!("Spec artifact is empty"));
	}
	let prompt_template = std::fs::read_to_string(SDLC_PROMPT_PLAN)?;
	let template = std::fs::read_to_string(SDLC_TEMPLATE_PLAN)?;

	let prompt = prompt_template
		.replace("{template}", &template)
		.replace("{intent}", intent)
		.replace("{spec}", spec);

	Ok(prompt)
}
pub fn plan_gen(intent: &str, spec: &str) -> String {
	format!(
		r#"
You are creating an implementation plan for an SDLC system.

The user's intent is authoritative.

## Intent

{intent}

## Specification

{spec}

## Instructions

Create a concrete implementation plan.

The plan must:
- identify the implementation work required
- break the work into ordered steps
- identify files/components likely to change
- identify dependencies between steps
- identify how each requirement will be verified
- avoid inventing requirements not present in the intent or specification

Return only the contents of `plan.md`.
"#,
	)
}
pub fn revise_spec(intent: &str, current_spec: &str, evaluation: &QACheck) -> Result<String> {
	Ok(format!(
		r#"Revise the existing specification so that it meets the requirements of the evaluator.

## Intent

{intent}

## Current Specification

{current_spec}

## Evaluation

Score: {score}
Confidence: {confidence}
Passed: {passed}

{evaluations}

## Instructions

Revise the specification to address the evaluator's failures.

Preserve correct information from the existing specification.

Do not merely describe the changes.
Produce the complete revised specification.

The output will replace spec.md directly.
"#,
		score = evaluation.score,
		confidence = evaluation.confidence,
		passed = evaluation.passed,
		evaluations = evaluation
			.evaluations
			.iter()
			.map(|e| {
				format!(
					"- {}: passed={}, score={}, confidence={}\n  {}",
					e.name, e.passed, e.score, e.confidence, e.explanation
				)
			})
			.collect::<Vec<_>>()
			.join("\n"),
	))
}
pub fn tests_gen(intent: &str, spec: &str, plan: &str) -> String {
	format!(
		r#"
			You are designing the verification plan for an SDLC task.

			The user's intent is authoritative.

			## Intent

			{intent}

			## Specification

			{spec}

			## Implementation Plan

			{plan}

			## Instructions

			Create `tests.md`.

			Every requirement in the specification must have at least
			one corresponding verification test.

			Tests should distinguish between:

			1. deterministic checks
				- cargo test
				- cargo check
				- cargo clippy
				- cargo fmt
				- application-specific commands

			2. behavioral tests
				- unit tests
				- integration tests
				- end-to-end tests

			3. semantic verification
				- requirements that cannot be established purely through
					deterministic commands and should later be evaluated by JEV

			Each test must be concrete enough that another agent can implement
			or execute it.

			Use this format:

			# Tests

			## Requirement: <requirement>

			- [ ] <test>

			Do not mark any test as complete.

			Return only the contents of `tests.md`.
		"#,
	)
}
