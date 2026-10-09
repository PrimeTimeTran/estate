# Plan: [Short Title of Implementation Strategy]

## 1. Overview

- **Objective:** [What this plan will accomplish]
- **Source Intent:** `intent/[slug].md`
- **Source Specification:** `spec/[slug].md`
- **Expected Outcome:** [What successful implementation looks like]

## 2. Repository Context

- **Repository Root:** [Verified absolute repository path]
- **Baseline Branch:** [Branch name, if known]
- **Baseline Commit:** [Commit hash, if known]
- **Relevant Existing Architecture:** [Components, modules, services, or entry points relevant to this task]

Use verified repository information wherever possible. Mark unknown values as `Unknown` rather than inventing them.

## 3. Expected File Changes

Identify **every file expected to be created, modified, or deleted** to complete the user's task.

This section is the authoritative file-change inventory for the implementation.

### 3.1 Files to Create

List each new file the implementation requires.

| File Path              | Purpose                   | Requirements Addressed            |
| ---------------------- | ------------------------- | --------------------------------- |
| `path/to/new_file.ext` | [Why this file is needed] | [Requirement IDs or descriptions] |

If no new files are required, state `None`.

### 3.2 Files to Modify

List each existing file that must change.

| File Path                   | Current Responsibility   | Expected Change    | Requirements Addressed            |
| --------------------------- | ------------------------ | ------------------ | --------------------------------- |
| `path/to/existing_file.ext` | [What it currently does] | [What must change] | [Requirement IDs or descriptions] |

If no existing files need modification, state `None`.

### 3.3 Files to Delete

List files that must be removed as part of the approved task.

| File Path                   | Reason for Removal          | Dependencies or References to Update                      |
| --------------------------- | --------------------------- | --------------------------------------------------------- |
| `path/to/obsolete_file.ext` | [Why deletion is necessary] | [Affected references, imports, configuration, or callers] |

If no files need deletion, state `None`.

### File Inventory Rules

- Derive file changes from the user's original goal, approved Intent, Specification, and verified repository context.
- Use repository-relative paths rooted at the verified repository root.
- Distinguish existing files from proposed new files.
- Inspect the repository before naming existing files to modify or delete.
- Prefer extending existing components over creating redundant files when appropriate.
- Include tests, configuration, documentation, and other supporting files when they actually need to change.
- Do not list files merely because they are related to the task.
- Do not invent paths, modules, APIs, or tests.
- If a required path cannot be determined without further inspection, mark it `Requires repository inspection` and explain what must be checked.
- Every listed file must have a clear purpose tied to the task or an applicable requirement.

## 4. Proposed Implementation

Describe how the files in Section 3 will be changed to fulfill the Specification.

### 4.1 [Component or Change Name]

- **Files Involved:** `path/to/file.ext`
- **Action:** [Create, Modify, or Delete]
- **Purpose:** [Why this change is needed]
- **Implementation Details:**
  - [Structural changes]
  - [Behavior or logic changes]
  - [Relevant type, API, or interface changes]
  - [Integration with other components]
- **Dependencies:** [Prerequisites or dependent changes]
- **Requirements Addressed:** [Requirement IDs or descriptions]

Repeat this subsection for each meaningful implementation change.

Do not introduce a file change here unless it is also recorded in Section 3. If implementation analysis reveals another necessary file, update the file inventory and its justification.

## 5. Verification and Testing Strategy

### 5.1 Unit Tests

- [ ] **Test Location:** `path/to/test_file.ext`
  - **Behavior Under Test:** [What is being verified]
  - **Expected Result:** [Observable success condition]
  - **Requirements Addressed:** [Requirement IDs or descriptions]

If no unit tests are applicable, explain why.

### 5.2 Integration and End-to-End Tests

- [ ] **Command or Procedure:** [Exact command or reproducible steps]
  - **Expected Result:** [What should happen]
  - **Requirements Addressed:** [Requirement IDs or descriptions]

Include runtime checks, CLI execution, application behavior, or other integration checks when relevant.

### 5.3 Regression Checks

- [ ] [Existing behavior that must remain functional]
- [ ] [Compatibility or integration boundary to verify]

### 5.4 Risks and Constraints

- **Risk:** [Potential regression or implementation concern]
- **Impact:** [What could go wrong]
- **Mitigation or Verification:** [How the risk will be addressed]

Include unresolved decisions that could block implementation. Do not silently resolve decisions reserved for the user or product owner.

## 6. Execution Work Order

List the implementation steps in dependency order.

### Step 1: [Action Name]

- **Files:** `path/to/file.ext`
- **Action:** [What to create, modify, or delete]
- **Prerequisites:** [Required preceding work]
- **Verification:** [How to confirm this step succeeded]

### Step 2: [Action Name]

- **Files:** `path/to/another_file.ext`
- **Action:** [What to implement]
- **Prerequisites:** [Required preceding work]
- **Verification:** [How to confirm this step succeeded]

Continue until all planned changes and required verification are complete.

The work order must use the file inventory in Section 3 and reflect actual dependencies. Do not invent additional scope or simply repeat the inventory without describing the execution sequence.

## 7. Completion Criteria

The implementation is ready for completion when:

- [ ] Every approved in-scope requirement has been addressed.
- [ ] Every planned file change has been implemented or explicitly reported as blocked.
- [ ] Tests and verification checks have been executed as planned, with results recorded.
- [ ] Relevant regressions and integration boundaries have been checked.
- [ ] No unapproved changes or unexplained file additions have been introduced.
- [ ] Any remaining limitations or unresolved decisions are documented.

## 8. Final File-Change Audit

Before finalizing the plan, verify:

- [ ] Every file to create, modify, or delete is listed in Section 3.
- [ ] Every listed file has a corresponding implementation action in Section 4 or a deletion action.
- [ ] Every implementation action references files in Section 3.
- [ ] File paths are verified or explicitly marked as requiring inspection.
- [ ] Test files are included when new or modified tests are needed.
- [ ] All changes trace back to the user's goal or an applicable Specification requirement.

Return only the completed Implementation Plan as Markdown.
