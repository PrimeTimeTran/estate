use super::*;

const INTENT_PROMPT: &str = include_str!("../../../ai/template/INITIAL_PROMPT.md");
const PROMPT_FROM_USER: &str = include_str!("../../../ai/template/user.goal.md");

pub fn for_intent(user_request: &str) -> String {
	INTENT_PROMPT.replace("{{PROMPT_FROM_USER}}", user_request)
}

pub fn gen_intent(goal: &str) -> Result<String> {
	Ok(format!(
		r#"
				You are defining the intent for an SDLC task.
				The user's goal is authoritative.

				## User Goal
				{goal}

				## Instructions
				Create `intent.md`.
				Describe what the user is trying to accomplish and why.

				The intent should:
				- preserve the user's actual goal without changing its meaning
				- describe the desired outcome
				- establish the problem or need being addressed
				- identify the important constraints explicitly stated by the user
				- avoid inventing requirements that the user did not state
				- remain implementation-independent where possible

				Do not write the specification, implementation plan, or tests yet.

				Use this format:

				# Intent <one sentence title summary for the goal>

				## Goal

				<what the user wants to accomplish>

				## Why

				<why this work is needed>

				## Constraints

				- <constraint>

				## Outcome

				<what successful completion should accomplish>

				Return only the contents of `intent.md`.
				"#,
	))
}
pub fn gen_spec(intent: &str) -> Result<String> {
	if intent.trim().is_empty() {
		return Err(anyhow!("cannot generate Spec prompt from empty Intent"));
	}

	Ok(format!(
		r#"
       	You are the Specification stage of an SDLC pipeline.

       	Your job is to transform the approved Intent artifact below into a concrete,
       	implementation-independent Specification.

       	You are NOT implementing the feature.
       	You are NOT writing source code.
       	You are NOT creating a plan.
       	You are NOT merely summarizing the Intent.

       	You are defining WHAT must be built, the boundaries of the work, the
       	constraints that apply, the expected system behavior, and the important
       	architectural concerns that must be resolved before implementation.

       	The resulting document will be written directly to:
          spec.md

       	Therefore, your entire response MUST be the specification document itself.
       	Do not include commentary before or after the specification.
       	Do not wrap the document in a Markdown code fence.

       	The Specification MUST use exactly this structure:

       	# Specification: [Feature or Project Name]

       	## 1. Overview & Inherited Intent

       	- **Source Intent:** intent.md
       	- **Core Objective:** [Brief summary of what this specification builds,
          explicitly inheriting the approved outcome from the Intent]

       	## 2. Requirements & Functional Scope

       	- **In-Scope:**
          - [Core capability 1]
          - [Core capability 2]

       	- **Out-of-Scope:**
          - [Explicit boundary / what is deferred]

       	Requirements must describe observable or verifiable behavior where possible.
       	Do not invent requirements that contradict the Intent.
       	If the Intent leaves something unspecified, identify that uncertainty rather
       	than silently inventing a product decision.

       	## 3. Policy & Governance Constraints (Applied Skills)

       	- **Brand & UX Guidelines:** [Applicable constraints, or "None identified"]
       	- **Security & Compliance:** [Applicable data handling, access control,
          privacy, security, and boundary constraints, or "None identified"]

       	Do not invent organizational policies.
       	Only state constraints supported by the Intent, existing project context,
       	or explicitly applicable system/project rules.

       	## 4. Proposed Design & Architecture

       	- **System Impact:** [Affected components, modules, services, files,
          persistence, APIs, integrations, or runtime boundaries]

       	- **User Experience Flow:** [Expected user-visible behavior and interaction
          flow, if applicable]

       	Describe the proposed system behavior and architecture at the level needed
       	for implementation to begin later.

       	Do NOT write implementation code.
       	Do NOT turn this section into an implementation plan.
       	Do NOT prescribe arbitrary technologies unless required by the existing
       	project context or the Intent.

       	## 5. Flagged Areas of Concern & Conflicts

       	- [Potential technical, UX, security, compatibility, performance, or
          architectural concern]
       	- [Unresolved contradiction or product decision requiring human input]

       	If no concerns or conflicts are identified, explicitly state:

       	- None identified.

       	CRITICAL RULES:

       	1. The Intent is the source of truth for the desired outcome.
       	2. Preserve the Intent's objective when converting it into requirements.
       	3. Separate requirements from implementation details.
       	4. Explicitly define both scope and boundaries.
       	5. Surface ambiguity instead of inventing decisions.
       	6. Surface conflicts instead of resolving product-policy conflicts yourself.
       	7. The Specification must be useful to a later Plan/Build stage.
       	8. The output must be a complete Markdown specification.
       	9. Do not discuss this prompt or your role.
       	10. Do not output anything except the completed specification.

       	Here is the approved Intent:

       	---

       	{intent}

       	---

       	Now produce the complete Specification.
     	"#,
	))
}
pub fn gen_plan(intent: &str, spec: &str) -> Result<String> {
	if intent.trim().is_empty() {
		return Err(anyhow!("cannot generate Plan prompt from empty Intent"));
	}
	if spec.trim().is_empty() {
		return Err(anyhow!("cannot generate Plan prompt from empty Spec"));
	}
	Ok(format!(
		r#"
       	You are the Plan stage of an SDLC pipeline.

       	Your job is to transform the approved Intent and Specification into a
       	concrete, repository-aware Implementation Plan.

       	You are NOT implementing the feature.
       	You are NOT writing source code.
       	You are NOT changing files.
       	You are NOT merely summarizing the Specification.

       	You are determining HOW the approved Specification should be implemented.

       	The resulting document will be written directly to:

            plan.md

       	Therefore, your entire response MUST be the implementation plan itself.
       	Do not include commentary before or after the plan.
       	Do not wrap the document in a Markdown code fence.

       	# REQUIRED OUTPUT STRUCTURE

       	Your response MUST follow this structure:

       	# Implementation Plan: [One-Sentence Strategy Summary]

       	The title MUST be a single sentence summarizing the primary
       	implementation strategy or technique that this plan will use.

       	The title should describe HOW the work will be accomplished, not merely
       	repeat the feature name.

       	For example:

       	# Implementation Plan: Introduce a normalized event pipeline that separates raw HID observation from semantic action dispatch.

       	Do not use generic titles such as:

       	- Implementation Plan: Keyboard Support
       	- Implementation Plan: New Feature
       	- Implementation Plan: Fix Bug

       	The title should communicate the central technical strategy.

       	## Overview

       	Briefly describe what this plan accomplishes and how it directly implements
       	the approved Specification.

       	The Overview must establish the relationship:

            Intent → Specification → Implementation Plan

       	Do not introduce functionality that is absent from the Specification.

       	## Context & References

       	- **Intent Reference**: `intent.md`
       	- **Specification Reference**: `spec.md`
       	- **Target Repository State**: [Current branch / relevant baseline]

       	Use the actual repository context available to you when known.

       	## Proposed Changes

       	List the precise files to create, modify, or delete.

       	For every meaningful implementation change, identify the concrete file path
       	and the action that will occur there.

       	Use this structure:

       	### [COMPONENT / MODULE NAME]

       	- **File Path**: `path/to/file.ext`
       	- **Action**: [Create | Modify | Delete]
       	- **Description of changes**:
          - Describe the structural changes.
          - Describe the logic changes.
          - Describe relevant API/type/interface changes.
          - Describe how this file connects to the rest of the implementation.

       	Do NOT invent arbitrary files.

       	Prefer existing repository files when they already provide the appropriate
       	extension point.

       	If the repository structure is not available, clearly identify the path as
       	requiring repository inspection rather than pretending that a path is known.

       	The plan must be specific enough that another agent or developer can execute
       	it without having to rediscover the architecture from scratch.

       	## Verification & Testing Strategy

       	Describe how the implementation will be verified.

       	### Unit Tests

       	Identify concrete tests to add or modify.

       	Use:

       	- [ ] Add/update test cases in `path/to/test.ext`
       	- [ ] Verify [specific behavior]

       	Tests should correspond directly to requirements in `spec.md`.

       	### Integration / End-to-End Checks

       	Identify integration tests, runtime checks, commands, or manual verification
       	needed to demonstrate that the implementation works.

       	Use concrete verification mechanisms where known.

       	For example:

       	- [ ] Run `cargo test`
       	- [ ] Run the relevant binary
       	- [ ] Verify the event flow produces the expected action
       	- [ ] Verify behavior in the affected application/runtime

       	Do not claim a test exists if it has not been identified.

       	### Expected Constraints/Risks

       	Identify potential regressions, compatibility issues, architectural risks,
       	performance concerns, policy boundaries, or unresolved assumptions.

       	Every significant concern should be connected to something identified in the
       	Specification.

       	## Execution Work Order

       	Provide an ordered sequence of implementation steps for an agent or human.

       	Each step must identify:

       	1. What is being changed.
       	2. Where it is being changed.
       	3. What dependency or prerequisite it has.
       	4. How it should be verified before proceeding.

       	Example:

       	1. Establish the new abstraction in `path/to/file.rs` and verify its unit tests.
       	2. Integrate the abstraction into `path/to/module.rs`.
       	3. Update dependent callers and tests.
       	4. Run the integration checks.
       	5. Perform the final regression suite.

       	The work order must reflect actual dependencies between changes.

       	Do not simply repeat the Proposed Changes section.

       	# PLANNING RULES

       	1. The Specification is the source of truth for WHAT must be built.
       	2. The Plan defines HOW that Specification will be implemented.
       	3. Do not expand scope beyond the Specification.
       	4. Do not resolve unresolved product decisions from the Specification by
          silently choosing one.
       	5. Identify unresolved decisions as risks or blockers.
       	6. Prefer the existing architecture and extension points over unnecessary
          new abstractions.
       	7. Identify concrete files, modules, types, interfaces, and tests whenever
          repository information makes that possible.
       	8. Do not write implementation code.
       	9. Do not produce pseudo-code as a substitute for a plan.
       	10. Do not produce generic advice.
       	11. Every proposed change must have a reason tied to the Specification.
       	12. Every verification step must prove a specific requirement or behavior.
       	13. The Execution Work Order must be actionable by another agent.
       	14. The final response must be a complete Markdown document.
       	15. Output ONLY the Implementation Plan.

       	# APPROVED INTENT

       	---

       	{intent}

       	---

       	# APPROVED SPECIFICATION

       	---

       	{spec}

       	---

       	Now produce the complete Implementation Plan.
      "#,
	))
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
pub fn revise_spec(
	intent: &str,
	current_spec: &str,
	evaluation: &StageEvaluation,
) -> Result<String> {
	Ok(format!(
		r#"
	Revise the existing specification so that it meets the requirements
	of the evaluator.

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