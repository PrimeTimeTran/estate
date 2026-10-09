# Plan: [Feature or Change Name]

## Overview

Briefly describe what this plan accomplishes and how it directly addresses the user's goal and `spec.md`.

## Files to Create or Modify

Identify the concrete files that need to be created, modified, or deleted to accomplish the user's requested task.

For each file, specify:

- **File Path**: Repository-relative path.
- **Action**: Create, Modify, or Delete.
- **Purpose**: Why this file needs to change and how it contributes to the user's goal.
- **Requirements Addressed**: The requirements from `spec.md` that necessitate this change, when available.

Rules:

- Derive the required file changes from the user's original prompt, goal, approved Intent, and Specification.
- Inspect the available repository context before identifying existing files to modify.
- Prefer modifying existing files over creating redundant files when appropriate.
- Do not invent file paths, modules, APIs, or tests.
- Distinguish verified existing files from proposed new files.
- If a file path or required change cannot be determined without further inspection, explicitly flag it.
- Include test files, configuration files, documentation, and other supporting files when they need to change.
- Do not include files merely because they are related to the task; every listed change must have a clear justification.
- Ensure the proposed file changes collectively address the user's goal and all applicable in-scope requirements.

## Context & References

- **User Goal**: [Original task or goal supplied by the user]
- **Intent Reference**: `intent/[slug].md`
- **Specification Reference**: `spec/[slug].md`
- **Target Repository State**: [Branch name / commit hash baseline]
- **Repository Root**: [Absolute or verified repository root]

## Proposed Changes

Describe the implementation details for the files identified in **Files to Create or Modify**.

### [Component / Module Name]

- **File Path**: `path/to/file.ext`
- **Action**: Create, Modify, or Delete
- **Description of Changes**:
  - Describe structural changes.
  - Describe logic or behavior changes.
  - Describe relevant API, type, or interface changes.
  - Explain how the change connects to other components.
  - Identify dependencies and prerequisites.

Do not introduce additional file changes here without also adding them to the **Files to Create or Modify** section and justifying their purpose.

## Verification & Testing Strategy

### Unit Tests

- [ ] Add or update tests in `path/to/test.ext`.
- [ ] Verify each relevant requirement from `spec.md`.
- [ ] Identify expected results and failure conditions.

### Integration / End-to-End Checks

- [ ] Identify integration tests, runtime checks, commands, or manual verification steps.
- [ ] Specify the expected observable behavior for each check.
- [ ] Verify that the implementation works within the affected application or runtime.

### Expected Constraints / Risks

- Identify potential regressions, compatibility issues, architectural risks, and policy boundaries.
- Identify unresolved decisions or assumptions that could block implementation.
- Connect each significant risk to the relevant requirement or proposed change.

## Execution Work Order

Provide an ordered sequence of steps for an agent or human to execute.

Each step must identify:

1. **Action**: What needs to be done.
2. **Files**: Which files from the proposed change list are involved
