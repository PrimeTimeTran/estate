# Specification

## Intent

Create a file named `hello-world.js` in the repository root. It should accept a command-line argument and write that value to `hello-world.md`.

The user should be able to run:

```bash
node hello-world.js "hi"
```

and have `hello-world.md` contain the supplied value.

## Requirements

* Create `hello-world.js` at the repository root.
* Accept the first command-line argument as the input value.
* Write the supplied value to `hello-world.md` at the repository root.
* Running `node hello-world.js "hi"` must produce a `hello-world.md` file containing `hi`.
* The implementation must be testable without requiring interactive input.
* Add tests covering:

  * The JavaScript logic for writing the supplied value.
  * The CLI behavior when invoked through Node.js.
* Tests must verify the generated file contents.
* Tests must clean up generated files or use an isolated temporary directory so they do not depend on state left by previous test runs.
* Verification must provide deterministic evidence that:

  * `hello-world.js` exists.
  * The JavaScript logic behaves correctly.
  * The CLI accepts an argument and writes the expected value.
  * The resulting `hello-world.md` contains exactly the expected value.
