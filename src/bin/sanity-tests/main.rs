use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use estate::{Agent, sdlc::*};

#[derive(Debug)]
struct Args {
	native: bool,
	prompts: Vec<PathBuf>,
}

// smoke-suite
// cargo run --bin sanity-tests -- --native src/bin/sanity-tests/tools-intern/*.md
// cargo run --bin sanity-tests -- --native src/bin/sanity-tests/tools-extern/*.md
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
	let args = parse_args()?;

	println!("native: {}", args.native);

	let generator = LocalGenerator::new(Agent::new(), "qwen3:8b");

	run_prompt_files(&generator, &args.prompts).await?;

	Ok(())
}

async fn run_prompt_files(generator: &LocalGenerator, paths: &[PathBuf]) -> Result<()> {
	for path in paths {
		run_prompt_file(generator, path).await?;
	}
	Ok(())
}
async fn run_prompt_file(generator: &LocalGenerator, path: &Path) -> Result<()> {
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
