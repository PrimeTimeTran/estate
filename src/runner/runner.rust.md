1. Dockerfile

```dockerfile
FROM rust:1.89-bookworm
RUN useradd --uid 1000 --create-home runner
WORKDIR /run
USER runner
ENTRYPOINT ["rustc"]
```

2. runner.sh

```runner.rust.sh
#!/bin/sh
set -e
rustc /run/solution.rs \
-O \
-o /run/solution
exec /run/solution
```

3. Run command

```sh
docker build -t leetcode-rust .
```

```rust
async fn run_rust(run: &Run) -> anyhow::Result<RunTelemetry> {
	let source = r#"
fn main() {
	println!("hello rust");
}
"#;

	let setup_start = Instant::now();

	let source_path = run.dir.join("solution.rs");

	tokio::fs::write(&source_path, source).await?;

	let setup_ms = setup_start.elapsed().as_millis();

	let execution_start = Instant::now();

	let output = tokio::time::timeout(
		Duration::from_secs(5),
		tokio::process::Command::new("docker")
			.args([
				"run",
				"--rm",
				"--network=none",
				"--read-only",
				"--cpus=1",
				"--memory=256m",
				"--pids-limit=64",
				"--cap-drop=ALL",
				"--security-opt=no-new-privileges",
				"--user=1000:1000",
			])
			.args([
				"-v",
				&format!("{}:/run:rw", run.dir.display()),
			])
			.args([
				"leetcode-rust",
			])
			.output(),
	)
	.await??;

	let execution_ms = execution_start.elapsed().as_millis();

	Ok(RunTelemetry {
		language: Language::Rust,
		setup_ms,
		compile_ms: 0,
		execution_ms,
		exit_code: output.status.code(),
		stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
		stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
	})
}
```
