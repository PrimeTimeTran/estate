# Implementation Plan: [Short Title]

## 1. Objective & Traceability

- **Objective:** [What this implementation will accomplish]
- **Source Intent:** [Reference to the active Intent artifact supplied by the SDLC runtime]
- **Source Specification:** [Reference to the active Specification artifact supplied by the SDLC runtime]
- **Expected Outcome:** [Observable result when implementation is complete]
- **Requirements Addressed:** [Requirement IDs from the Specification]

The Intent and Specification are the authoritative sources for the task's goals, scope, and requirements. This plan translates them into a concrete, verifiable implementation strategy.

Do not invent requirements, silently expand scope, or resolve decisions reserved for the user or product owner.

## 2. Verified Repository Context

Use repository and environment information supplied by the runtime, and inspect the repository when necessary.

- **Repository Root:** [Verified absolute repository path]
- **Baseline Branch:** [Verified branch name, or Unknown]
- **Baseline Commit:** [Verified commit hash, or Unknown]
- **Working Tree State:** [Relevant pre-existing changes, or Unknown]
- **Relevant Architecture:** [Existing components, modules, services, entry points, and dependencies]
- **Inspection Performed:** [Relevant files, symbols, configuration, and tests inspected]

### Context Rules

- Treat the runtime-provided workspace root as the authoritative repository boundary.
- Use repository-relative paths for files inside the repository.
- Verify that existing files, modules, APIs, and tests actually exist before referencing them as existing.
- Distinguish verified facts from assumptions and inferences.
- Mark unknown information as `Unknown` rather than inventing values.
- Respect repository instructions, applicable agent policies, and existing architectural constraints.
- Do not assume that the current working directory, artifact directory, or repository root are interchangeable.

## 3. Authoritative File-Change Inventory

Identify every repository file expected to be created, modified, or deleted to fulfill the approved task.

This section is the authoritative inventory for implementation scope. Every implementation action in Section 4 and every work-order step in Section 5 must reference files listed here.

### 3.1 Files to Create

| File Path | Purpose | Requirements Addressed |
|---|---|---|
| `path/to/new_file.ext` | [Why the file is necessary] | [Requirement IDs] |

If no new files are required, state `None`.

### 3.2 Files to Modify

| File Path | Current Responsibility | Expected Change | Requirements Addressed |
|---|---|---|---|
| `path/to/existing_file.ext` | [Verified current behavior] | [Specific proposed change] | [Requirement IDs] |

If no existing files need modification, state `None`.

### 3.3 Files to Delete

| File Path | Reason for Removal | Dependencies or References to Update |
|---|---|---|
| `path/to/obsolete_file.ext` | [Why deletion is necessary] | [Verified affected references or dependencies] |

If no files need deletion, state `None`.

### File Inventory Rules

- Derive changes from the user's original goal, approved Intent, Specification, and verified repository context.
- Include only files needed to implement the approved scope.
- Inspect the repository before identifying existing files for modification or deletion.
- Distinguish existing files from proposed new files.
- Include tests, documentation, configuration, migrations, and other supporting files when they actually need to change.
- Prefer extending existing components over introducing redundant abstractions.
- Do not invent file paths, APIs, modules, dependencies, or tests.
- If a path cannot be established without further inspection, use `Requires repository inspection` and explain what must be verified.
- If a decision prevents a reliable inventory, identify it as an unresolved decision rather than fabricating a solution.
- Every file must have a clear purpose and traceability to the approved task.
- Do not include generated build artifacts or temporary files unless they are explicitly required deliverables.

## 4. Proposed Implementation

Describe the concrete changes needed to fulfill the Specification. Organize the work by meaningful component or behavior, not by arbitrary file groupings.

### 4.1 [Component or Change Name]

- **Files:** [Paths from Section 3]
- **Action:** [Create, Modify, or Delete]
- **Purpose:** [Why the change is necessary]
- **Requirements Addressed:** [Requirement IDs]
- **Implementation Details:**
  - [Structural changes]
  - [Behavior and logic changes]
  - [Relevant types, interfaces, APIs, or data changes]
  - [Integration with existing components]
  - [Compatibility considerations]
- **Dependencies:** [Prerequisite changes or external dependencies]
- **Verification:** [Observable behavior or checks that demonstrate correctness]

Repeat this subsection for each meaningful implementation change.

### Implementation Rules

- Do not introduce unapproved scope or unnecessary architectural complexity.
- Follow existing repository conventions unless a justified change is required.
- Reuse existing types, APIs, and abstractions where appropriate.
- Do not assume an API, function, module, or dependency exists without verification.
- Do not introduce a file change here unless it is recorded in Section 3.
- If implementation analysis reveals another necessary file, update Section 3 and its justification before proceeding.
- If the proposed design conflicts with a mandatory constraint or requires an unresolved product decision, document the conflict instead of silently choosing a resolution.

## 5. Execution Work Order

List the implementation steps in dependency order. Each step must reference the authoritative file inventory in Section 3.

### Step 1: [Action Name]

- **Files:** [Paths from Section 3]
- **Action:** [Specific implementation work]
- **Prerequisites:** [Required preceding steps, or None]
- **Requirements Addressed:** [Requirement IDs]
- **Verification:** [Command, test, inspection, or observable result]

### Step 2: [Action Name]

- **Files:** [Paths from Section 3]
- **Action:** [Specific