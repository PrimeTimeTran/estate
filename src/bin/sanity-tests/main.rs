use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use estate::{
	new_agent_system,
	sdlc::{traits::ArtifactGenerator, *},
};

#[derive(Debug)]
struct Args {
	native: bool,
	prompts: Vec<PathBuf>,
}

// smoke-suite
// cargo run --bin sanity-tests -- --native src/bin/sanity-tests/tools-decide_next_action/*.md
// cargo run --bin sanity-tests -- --native src/bin/sanity-tests/tools-host-env/*.md
fn parse_args() -> Result<Args> {
	let mut native = false;
	let mut prompts = Vec::new();
	for arg in std::env::args_os().skip(1) {
		if arg == "--native" {
			native = true;
		} else {
			prompts.push(PathBuf::from(arg));
		}
	}

	if prompts.is_empty() {
		bail!("usage: cargo run --bin ai-evaluator -- --native <prompt.md> [prompt.md ...]");
	}

	Ok(Args { native, prompts })
}

#[tokio::main]
async fn main() -> Result<()> {
	// 1. awe! — Section arm
	awe!("AGENT");

	// 2. awe! — Debug-value arm
	let count = 42;
	awe!(Info, count);

	// 3. awe! — Formatted-message arm
	awe!(Info, "Agent started with {} files", count);

	// 4. section! — Section macro
	section!("WORKSPACE");

	let args = parse_args()?;
	println!("native: {}", args.native);
	let (_bus, runtime, _event_rx) = new_agent_system();
	let generator = LocalGenerator::new(runtime, "qwen3:8b");
	run_prompt_files(&generator, &args.prompts).await?;
	Ok(())
}
async fn run_prompt_files(generator: &LocalGenerator, paths: &[PathBuf]) -> Result<()> {
	for path in paths {
		run_prompt(generator, path).await?;
	}
	Ok(())
}
async fn run_prompt(generator: &LocalGenerator, path: &Path) -> Result<()> {
	println!();
	println!("========================================");
	println!("PROMPT: {}", path.display());
	println!("========================================");
	let prompt = tokio::fs::read_to_string(path)
		.await
		.with_context(|| format!("reading prompt {}", path.display()))?;
	generator
		.run_agent(&prompt)
		.await
		.with_context(|| format!("running prompt {}", path.display()))?;
	Ok(())
}
