use super::*;

pub fn for_intent(user_request: &str) -> String {
	INTENT_PROMPT.replace("{{SDLC_GOAL}}", user_request)
}

pub fn gen_intent(goal: &str) -> Result<String> {
	if goal.trim().is_empty() {
		return Err(anyhow!("cannot generate Intent prompt from empty goal"));
	}
	let prompt = SDLC_PROMPT_INTENT
		.replace("{template}", SDLC_TEMPLATE_INTENT)
		.replace("{goal}", goal);
	Ok(prompt)
}

pub fn gen_spec(intent: &str) -> Result<String> {
	if intent.trim().is_empty() {
		return Err(anyhow!("cannot generate Spec prompt from empty Intent"));
	}
	let prompt = SDLC_PROMPT_SPEC
		.replace("{template}", SDLC_TEMPLATE_SPEC)
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
	let prompt = SDLC_PROMPT_PLAN
		.replace("{template}", SDLC_TEMPLATE_PLAN)
		.replace("{intent}", intent)
		.replace("{spec}", spec);
	Ok(prompt)
}

pub fn parse_completion_criteria(
	plan: &str,
) -> Result<crate::modules::native::agent::CompletionCriteria> {
	let lines = plan.lines().collect::<Vec<_>>();
	let contract_index = lines
		.iter()
		.position(|line| {
			let heading = line.trim().strip_prefix("##").map(str::trim).unwrap_or("");
			let title = heading
				.split_once(". ")
				.filter(|(number, _)| !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
				.map_or(heading, |(_, title)| title);
			title == "Machine-Readable Build Contract"
		})
		.ok_or_else(|| anyhow!("Plan is missing the Machine-Readable Build Contract section"))?;
	let contract = lines[contract_index + 1..].join("\n");
	let json = contract
		.trim_start()
		.strip_prefix("```json")
		.and_then(|contract| contract.split_once("```").map(|(json, _)| json.trim()))
		.ok_or_else(|| anyhow!("Build Contract must contain a fenced JSON object"))?;
	let criteria: crate::modules::native::agent::CompletionCriteria =
		serde_json::from_str(json).context("parsing Plan Build Contract")?;
	if criteria.required_files.is_empty() {
		return Err(anyhow!(
			"Build Contract must list at least one required output file"
		));
	}
	if criteria.required_commands.is_empty() {
		return Err(anyhow!(
			"Build Contract must list at least one verification command"
		));
	}
	for path in &criteria.required_files {
		if path.as_os_str().is_empty()
			|| path.is_absolute()
			|| path
				.components()
				.any(|component| matches!(component, std::path::Component::ParentDir))
		{
			return Err(anyhow!(
				"Build Contract contains an invalid output path: {}",
				path.display()
			));
		}
	}
	if criteria.required_commands.iter().any(|command| {
		command.is_empty() || command[0].trim().is_empty() || command.iter().any(String::is_empty)
	}) {
		return Err(anyhow!(
			"Build Contract contains an empty verification command"
		));
	}
	Ok(criteria)
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

#[cfg(test)]
mod completion_contract_tests {
	use super::*;

	#[test]
	fn parses_machine_readable_build_contract() {
		let plan = r#"## 9. Machine-Readable Build Contract
```json
{
  "required_files": ["hello-world.cjs", "tests/hello-world.test.js"],
  "required_commands": [["node", "--test", "tests/hello-world.test.js"]]
}
```
"#;
		let criteria = parse_completion_criteria(plan).unwrap();
		assert_eq!(criteria.required_files.len(), 2);
		assert_eq!(
			criteria.required_commands[0],
			["node", "--test", "tests/hello-world.test.js"]
		);
	}

	#[test]
	fn rejects_contract_paths_outside_the_workspace() {
		let plan = r#"## Machine-Readable Build Contract
```json
{"required_files":["../outside"],"required_commands":[["node","--test"]]}
```
"#;
		assert!(parse_completion_criteria(plan).is_err());
	}
}
