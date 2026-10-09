# Build a Node.js CLI Hello World Tool

Create a small Node.js CLI tool in the repository root.

## Goal

The tool should accept a single command-line argument and write that value to a Markdown file.

The CLI entrypoint must be:

```text
hello-world.cjs
```

The generated output file must be:

```text
hello-world.md
```

## Required behavior

From the repository root, the user must be able to run:

```bash
node hello-world.cjs "hi"
```

and the command must create or overwrite `hello-world.md` so that the file contains the supplied value.

For example:

```bash
node hello-world.cjs "hi"
```

must result in:

```text
hello-world.md
```

containing:

```text
hi
```

The implementation should use Node.js and should not require a third-party runtime or framework.

## Tests

Add automated tests covering both:

1. **JavaScript logic**
   - Test the logic responsible for taking an input value and producing/writing the expected Markdown output.
   - The tests should be able to exercise this logic without needing to invoke the CLI as a separate process.

2. **CLI behavior**
   - Test the actual command-line entrypoint.
   - The test should invoke the equivalent of:
     ```bash
     node hello-world.cjs "hi"
     ```
   - Verify that the expected `hello-world.md` file is created and contains the expected value.

Tests should avoid modifying unrelated repository files and should clean up any temporary files they create.

## Acceptance criteria

The implementation is complete when all of the following are true:

- `hello-world.cjs` exists at the repository root.
- `hello-world.cjs` is executable through Node.js with no additional runtime setup.
- Running `node hello-world.cjs "hi"` succeeds.
- Running the command creates or overwrites `hello-world.md` at the repository root.
- `hello-world.md` contains the supplied argument.
- The JavaScript logic has automated test coverage.
- The CLI behavior has automated test coverage.
- All automated tests pass.
- No unrelated existing files or behavior are modified.

Do not add functionality beyond this requirement unless it is necessary for testing or basic robustness.
