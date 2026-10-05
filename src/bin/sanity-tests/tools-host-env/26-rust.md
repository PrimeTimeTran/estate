Requirements:

- Create, update, or edit files only using paths under `00-sanity-tests/`; do not rely on changing directories.
- Your first shell command must be exactly:
  `mkdir -p 00-sanity-tests`
- Create `00-sanity-tests/26-rust.md`.
- Execute the requested inspection commands and write their actual results into the Markdown file.
- Capture **both stdout and stderr** from every command using `2>&1`.
- Do not summarize, paraphrase, or invent any command output.

The file must contain these sections:

# Rust Environment

## `rustc --version --verbose`

```text
<actual stdout and stderr>
```

## `cargo --version`

```text
<actual stdout and stderr>
```

## `rustdoc --version`

```text
<actual stdout and stderr>
```

## `rustfmt --version`

```text
<actual stdout and stderr>
```

## `rust-analyzer --version`

```text
<actual stdout and stderr>
```

## `cargo clippy --version`

```text
<actual stdout and stderr>
```

## `cargo fmt --version`

```text
<actual stdout and stderr>
```

## `rustup --version`

```text
<actual stdout and stderr>
```

## `rustup show active-toolchain`

```text
<actual stdout and stderr>
```

## `rustup target list --installed`

```text
<actual stdout and stderr>
```

Command construction requirements:

- The **first shell command** must be `mkdir -p 00-sanity-tests`.
- After that, execute the inspection as **one shell command**.
- Do not use `cd`.
- Do not create temporary files.
- Do not use separate independent shell invocations for each inspection.
- Do not rely on the shell command's final stdout being returned to the caller. The important result is the contents of `00-sanity-tests/26-rust.md`.
- Use a single grouped shell command such as:

````sh
{
  printf '%s\n' '# Rust Environment'
  printf '%s\n\n' '## `rustc --version --verbose`'
  printf '%s\n' '```text'
  rustc --version --verbose 2>&1
  printf '%s\n' '```'
  ...
} > 00-sanity-tests/26-rust.md
````

- Every inspection command must use `2>&1` so errors and informational messages are captured in the file.
- A failing inspection command must **not prevent the remaining inspections from running**. Do not use `&&` between the individual inspection commands.
- The initial `mkdir -p 00-sanity-tests` must succeed before attempting to create the file.
- After writing the file, verify it exists and inspect its contents.
- The final file should contain all requested sections, even when one of the commands fails.

Do not output only the raw version lines. The required deliverable is the Markdown file with a labeled section and fenced `text` code block for **every individual command**.
