# Task

Build a LeetCode-style problem-solving UI in:

/Users/future/kb/project/crates/estate

You must inspect the existing codebase and MODIFY the existing source files.
Do not merely describe or suggest code.

---

## Primary Goal

Implement the UI in:

./src/ui/view/problem.rs

The resulting UI should have:

1. Left sidebar
   - Tabbed navigation similar to LeetCode / VS Code.
   - Tabs should be clickable.
   - Selecting a tab changes the content displayed in the sidebar.
   - One tab should display the current problem description.

2. Main content area
   - Large central code-editor area.
   - This should occupy the majority of the available window.
   - The layout should feel like a coding/problem-solving IDE.

3. Overall layout
   - Sidebar on the left.
   - Code editor in the center.
   - Use egui layout primitives appropriate for a resizable desktop application.
   - Follow the existing application's visual/style conventions where possible.

---

## Existing Architecture

Before making changes, inspect these files:

### UI trait

./src/ui/view/ve.rs

Read:

pub trait Veable<R: Runtime>

Pay particular attention to:

- draw
- update
- event

These define how UI views are rendered and interacted with in this application.

### Target view

./src/ui/view/problem.rs

This is the primary file to modify.

Inspect the existing implementation and integrate the new UI into its existing architecture rather than replacing the architecture unnecessarily.

### Problem model

./src/model/problem.rs

Use this to understand the available problem data and types.

### Submission model

./src/model/submission.rs

Use this to understand submission-related data and types.

---

### Requirements

- Use the existing Rust/egui architecture.
- Use idiomatic Rust.
- Reuse existing types/components where appropriate.
- Do not introduce a second UI architecture.
- Do not create unnecessary abstractions.
- Preserve existing functionality unless modification is required for the new UI.
- The UI must compile.
- Prefer implementing the UI directly in `problem.rs` unless another file is genuinely required.
- If additional files are necessary, modify them as needed.

---

## Visual Direction

Use a layout inspired by LeetCode and VS Code:

```text
┌────────────────┬──────────────────────────────────────┐
│ Problem        │                                      │
│ Description    │                                      │
│ Solutions      │             CODE EDITOR              │
│ Submissions    │                                      │
│                │                                      │
│                │                                      │
│ ────────────── │                                      │
│ Problem title  │                                      │
│ Difficulty     │                                      │
│ Tags           │                                      │
└────────────────┴──────────────────────────────────────┘
```

The exact styling does not need to match either application.
Prioritize:

- clear hierarchy
- usable spacing
- clickable tabs
- obvious active-tab state
- responsive/resizable layout
- large editor area
- clean desktop IDE appearance

---

## Workflow

1. Inspect the relevant files and surrounding UI architecture.
2. Determine how `ProblemView` currently receives and stores its state.
3. Implement the sidebar/tab state.
4. Implement the sidebar contents.
5. Implement the main editor area.
6. Integrate the UI with the existing `Veable` lifecycle.
7. Run the appropriate Rust checks/build.
8. Fix compilation errors and integration issues.
9. Leave the working changes applied to the repository.

---

## Important

MAKE THE CHANGES DIRECTLY TO THE PROJECT.

Do not respond with a proposed patch or a code listing instead of modifying the files.

Do not stop after explaining what should be done.

Do not say "next", "complete", or merely describe the changes.

Your job is to inspect, implement, build/check, and fix the code.

The filesystem is the deliverable. Apply the implementation directly to the repository. Your final response should briefly summarize what you changed and any build/test result. Do not paste the implementation into the response.

## Scope

Implement the requested UI without redesigning unrelated parts of the application.

Do not:

- rewrite existing architecture unnecessarily
- introduce a new UI framework
- refactor unrelated modules
- replace existing models
- create placeholder infrastructure for features that are not requested

If an existing abstraction can support the UI, use it.
If the existing architecture prevents the requested UI, make the smallest reasonable architectural change.

---

## Task: Implement Problem UI

You are working directly in this repository:

`/Users/future/kb/project/crates/estate`

## REQUIRED OUTCOME

**Modify the repository. Do not just explain what you would do.**

Your task is not complete until you have:

1. Inspected the existing implementation.
2. Edited the source files.
3. Implemented the requested UI.
4. Run a Rust build/check.
5. Fixed any compilation errors caused by your changes.

**You MUST use your file/shell/editing tools to make the changes.**

The final response is NOT the deliverable. The modified files in the repository are the deliverable.

---

## UI TO BUILD

Modify:

`./src/ui/view/problem.rs`

Build a LeetCode/VS Code-style coding interface:

```text
┌────────────────────┬──────────────────────────────────────────┐
│ Problem             │                                          │
│ Description         │                                          │
│ Solutions           │             CODE EDITOR                  │
│ Submissions         │                                          │
│                     │                                          │
│ ──────────────────  │                                          │
│                     │                                          │
│ Problem information │                                          │
│                     │                                          │
└────────────────────┴──────────────────────────────────────────┘
```

### Left sidebar

Create a tabbed sidebar.

At minimum, provide tabs for:

- Problem
- Solutions
- Submissions

The tabs must be clickable and have visible active/inactive states.

The selected tab controls the content displayed in the sidebar.

The Problem tab should display the current problem's information/description using the existing problem model.

### Main area

Create a large central code-editor area.

Use the existing editor implementation if one already exists in the project. If no editor exists, create an appropriate egui-based editable area/placeholder consistent with the existing architecture.

The editor should occupy most of the available window.

---

## EXISTING ARCHITECTURE

Before editing, inspect these files:

### UI lifecycle

`./src/ui/view/ve.rs`

Read:

`pub trait Veable<R: Runtime>`

Pay particular attention to:

- `draw`
- `update`
- `event`

Follow the existing lifecycle and architecture.

### Target

`./src/ui/view/problem.rs`

This is the primary file to modify.

Understand its existing state and behavior before changing it.

### Models

`./src/model/problem.rs`

Use this to understand problem data.

`./src/model/submission.rs`

Use this to understand submission data.

Also inspect nearby UI/view code when useful so the new UI follows existing project conventions.

---

## IMPLEMENTATION RULES

- Use Rust and egui.
- Follow the existing `Veable` architecture.
- Reuse existing project abstractions where appropriate.
- Keep the implementation reasonably simple.
- Do not redesign unrelated parts of the application.
- Do not merely provide a suggested patch.
- Apply the changes directly to the repository.
- If additional files genuinely need modification, modify them.
- Do not stop after inspecting files.
- Do not stop after describing an implementation plan.

---

## REQUIRED WORKFLOW

Start by inspecting the relevant source files.

Then immediately begin implementing the UI.

After implementation, run the appropriate Cargo check/build for the project.

If compilation fails:

1. Read the errors.
2. Fix the errors.
3. Run the check/build again.
4. Continue until the changes compile or you encounter a genuine pre-existing blocker.

Do not ask me whether you should proceed.

Do not wait for further instructions.

**Proceed autonomously and make the changes now.**
