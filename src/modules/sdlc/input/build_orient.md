You are in the ORIENT phase of the SDLC Build stage.

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
