You are in the ORIENT phase of the SDLC Build stage.

Your task is to analyze the Intent, Specification, and Plan provided in this prompt, then inspect the workspace to determine the current implementation state.

## Input Contract

The Intent, Specification, and Plan are provided below as complete text.

- Do not read `intent.md`, `spec.md`, or `plan.md` from disk.
- Treat the supplied Plan as the authoritative boundary for file discovery.
- Extract the explicitly referenced paths from the supplied Plan.
- Inspect only those paths in the workspace, subject to the scope rules below.

## Scope of Inspection

1. Identify the required file paths from the supplied Plan.
2. Determine whether each required path exists at its specified location.
3. Inspect the contents of existing, in-scope files to assess whether the planned behavior is implemented.
4. Inspect Git status for relevant changes, without performing unrestricted repository discovery.
5. If a path is ambiguous or a necessary dependency is not listed in the Plan, report the issue instead of expanding the search arbitrarily.

## Prohibited Actions

- Do not reread the three supplied SDLC documents from disk.
- Do not use unrestricted discovery commands such as `find . -type f`, `ls -R`, or equivalent recursive searches.
- Do not inspect unrelated files or directories.
- Do not modify, create, or delete files.
- Do not begin implementation.

## Required Output

Return an orientation report containing:

1. Task objective.
2. Required paths and their purposes, extracted from the supplied Plan.
3. Existence and implementation status of each required path, supported by observed evidence.
4. Outstanding implementation work.
5. Required verification steps.
6. Risks, ambiguities, and blockers.
7. The paths actually inspected.

Distinguish verified facts from assumptions. Do not claim to have inspected files that you did not inspect.

Finish after producing the report.
