use super::*;
use std::fmt::Write;

use anyhow::Context as CtxAnyhow;

fn shell_quote(value: &str) -> String {
	if value.is_empty() {
		return "''".to_owned();
	}

	if value
		.chars()
		.all(|c| c.is_ascii_alphanumeric() || "_./:-=+".contains(c))
	{
		return value.to_owned();
	}

	format!("'{}'", value.replace('\'', "'\\''"))
}

pub fn format_changed_files(paths: &[std::path::PathBuf]) -> String {
	if paths.is_empty() {
		return "(no filesystem changes detected)".to_owned();
	}

	let mut paths = paths
		.iter()
		.map(|path| path.display().to_string())
		.collect::<Vec<_>>();

	paths.sort();
	paths.dedup();

	paths
		.into_iter()
		.map(|path| format!("- {path}"))
		.collect::<Vec<_>>()
		.join("\n")
}
pub fn validate_agent_completion(
	workspace_root: &Path,
	artifacts: &[Artifact],
	commands: &[CommandRecord],
	changed_paths: &[PathBuf],
) -> Result<AgentCompletionValidation> {
	let mut errors = Vec::new();
	let mut paths: Vec<PathBuf> = artifacts
		.iter()
		.filter_map(|artifact| match artifact {
			Artifact::FileRead { path, .. } | Artifact::FileWrite { path } => Some(PathBuf::from(path)),
			Artifact::Observation(_) | Artifact::ToolOutput(_) => None,
		})
		.collect();

	paths.extend(changed_paths.iter().cloned());
	paths.sort();
	paths.dedup();

	if !changed_paths.is_empty() && !commands.iter().any(|command| command.success) {
		errors.push(
			"Files changed, but no command completed successfully. \
			 Run an appropriate verification command before finishing."
				.to_owned(),
		);
	}

	Ok(AgentCompletionValidation { errors })
}
pub fn reconcile_agent_artifacts(
	ctx: &mut AgentCtx,
	before: &WSSnapshot,
	after: &WSSnapshot,
) -> Result<Vec<PathBuf>> {
	let changes = before.diff(after);
	let changed_paths = parse_git_status_paths(&changes.git_status);

	for path in &changed_paths {
		let absolute_path = if path.is_absolute() {
			path.clone()
		} else {
			ctx.workspace.cwd.join(path)
		};

		let path_string = absolute_path.to_string_lossy().into_owned();

		if absolute_path.is_file() {
			ctx.workspace.add_file(absolute_path.clone())?;

			let artifact = Artifact::FileWrite {
				path: path_string.clone(),
			};

			// Avoid duplicate write artifacts for the same path.
			if !ctx.artifacts.iter().any(|existing| {
				matches!(
					existing,
					Artifact::FileWrite { path } if path == &path_string
				)
			}) {
				ctx.artifacts.push(artifact);
			}
		} else {
			// Remove stale workspace entries for deleted files.
			ctx
				.workspace
				.files
				.retain(|file| Path::new(&file.path) != absolute_path);

			// Remove write artifacts for files that no longer exist.
			ctx.artifacts.retain(|artifact| {
				!matches!(
					artifact,
					Artifact::FileWrite { path } if path == &path_string
				)
			});
		}
	}

	Ok(changed_paths)
}
pub fn parse_git_status_paths(status: &str) -> Vec<PathBuf> {
	let mut paths = Vec::new();

	for line in status.lines() {
		// `git status --short` starts with two status columns.
		if line.len() < 4 {
			continue;
		}

		let status_code = &line[..2];
		let path = line[3..].trim();

		if path.is_empty() || status_code == "!!" {
			continue;
		}

		// For renames, porcelain output is generally "old -> new".
		let path = path
			.rsplit_once(" -> ")
			.map(|(_, new_path)| new_path)
			.unwrap_or(path)
			.trim_matches('"');

		paths.push(PathBuf::from(path));
	}

	paths.sort();
	paths.dedup();
	paths
}
pub fn validate_shell_command(command: &str) -> anyhow::Result<()> {
	use anyhow::bail;

	if command.trim().is_empty() {
		bail!("Command cannot be empty.");
	}

	if command.len() > 2_000 {
		bail!("Command exceeds the 2000-character limit.");
	}

	let chains =
		command.matches("&&").count() + command.matches("||").count() + command.matches(';').count();

	if chains > 2 {
		bail!(
			"Too many chained operations. Run one bounded operation \
			at a time and inspect its result."
		);
	}

	Ok(())
}

pub async fn build_action(prompt: &str) -> Result<LlmAction> {
	use std::time::Instant;

	let started = Instant::now();

	eprintln!("{}", format_kv("build_action:", "ENTER"));
	eprintln!("{}", format_kv("MODEL:", DEFAULT_MODEL));
	eprintln!("{}", format_kv("ENDPOINT:", crate::AGENT_GEN_URL));
	eprintln!("{}", format_kv("PROMPT CHARS:", &prompt.len().to_string()));
	eprintln!(
		"{}",
		format_kv(
			"SYSTEM PROMPT CHARS:",
			&JSON_PROMPT_EXECUTION.len().to_string()
		)
	);

	let payload = serde_json::json!({
		"model": DEFAULT_MODEL,
		"system": JSON_PROMPT_EXECUTION,
		"prompt": prompt,
		"stream": false,
		"format": "json"
	});

	let client = reqwest::Client::builder()
		.connect_timeout(std::time::Duration::from_secs(5))
		.timeout(std::time::Duration::from_secs(120))
		.build()
		.context("building HTTP client")?;

	eprintln!("{}", format_kv("HTTP:", "SENDING REQUEST"));
	let request_started = std::time::Instant::now();

	let res = client
		.post(crate::AGENT_GEN_URL)
		.json(&payload)
		.send()
		.await
		.with_context(|| {
			format!(
				"sending model request to {} after {:?}",
				crate::AGENT_GEN_URL,
				request_started.elapsed()
			)
		})?;

	eprintln!(
		"{}",
		format_kv(
			"HTTP RESPONSE AFTER:",
			&format!("{:?}", request_started.elapsed())
		)
	);
	eprintln!("{}", format_kv("HTTP STATUS:", &res.status().to_string()));

	// eprintln!(
	// 	"{}",
	// 	format_kv(
	// 		"HTTP:",
	// 		&format!("HEADERS RECEIVED after {:?}", request_started.elapsed())
	// 	)
	// );
	// eprintln!("{}", format_kv("HTTP STATUS:", res.status().as_str()));

	let status = res.status();

	eprintln!("{}", format_kv("HTTP BODY:", "READING"));
	let body_started = Instant::now();

	let body = res
		.text()
		.await
		.context("reading model HTTP response body")?;

	eprintln!(
		"{}",
		format_kv(
			"HTTP BODY:",
			&format!("{} chars, read in {:?}", body.len(), body_started.elapsed())
		)
	);

	if !status.is_success() {
		eprintln!(
			"{}",
			format_kv("HTTP ERROR BODY:", &truncate_output(&body, 2_000))
		);

		return Err(anyhow::anyhow!("Model endpoint returned HTTP {status}"));
	}

	eprintln!("{}", format_kv("HTTP JSON:", "PARSING"));

	let res: serde_json::Value =
		serde_json::from_str(&body).context("parsing model HTTP response JSON")?;

	let response = res["response"].as_str().unwrap_or("");

	section!("RAW MODEL ACTION RESPONSE");
	println!(
		"{}",
		format_kv("RESPONSE LENGTH:", &response.len().to_string())
	);
	println!("response:\n{response}");
	println!("end raw response");

	eprintln!("{}", format_kv("ACTION JSON:", "PARSING"));

	let mut raw: LlmAction = serde_json::from_str(response).context("parsing model action JSON")?;

	if raw.command.is_none() {
		if let Some(args) = raw.args.as_ref() {
			if !args.is_empty() {
				raw.command = Some(
					args
						.iter()
						.map(|arg| shell_quote(arg))
						.collect::<Vec<_>>()
						.join(" "),
				);
			}
		}
	}

	println!("PARSED ACTION:");
	println!("  action: {:?}", raw.action);
	println!("  command: {:?}", raw.command);
	println!("  message: {:?}", raw.message);

	eprintln!(
		"{}",
		format_kv("build_action TOTAL:", &format!("{:?}", started.elapsed()))
	);
	eprintln!("{}", format_kv("build_action:", "RETURN"));

	Ok(raw)
}
pub fn build_prompt(run: &AgentRun) -> String {
	let ctx = &run.ctx;

	eprintln!("{}", format_kv("build_prompt:", "1️⃣ ENTER"));

	let workspace = format_workspace(&ctx.workspace);
	let history = format_history(&ctx.history, 3, 3);
	let execution_ledger = format_execution_ledger(&run.commands, 5, 5, 4_000);
	let plan_path = "/Users/future/kb/project/crates/estate/log/plan.md";

	let plan = match std::fs::read_to_string(plan_path) {
		Ok(content) => content,
		Err(error) => {
			eprintln!("{}", format_kv("PLAN READ ERROR:", &error.to_string()));
			format!("ERROR: Could not read required plan.md at {plan_path}: {error}")
		}
	};

	eprintln!(
		"{}",
		format_kv("CWD:", &ctx.workspace.cwd.to_str().unwrap())
	);
	eprintln!("{}", format_kv("PLAN PATH:", plan_path));
	eprintln!(
		"{}",
		format_kv(
			"TASK CHARS:",
			&ctx.prompt.as_deref().unwrap_or("").len().to_string()
		)
	);

	eprintln!("{workspace}");
	eprintln!("{history}");
	eprintln!("{execution_ledger}");

	let prompt = ACTION_PROMPT_EXECUTION
		.replace("{task}", ctx.prompt.as_deref().unwrap_or(""))
		.replace("{workspace}", &workspace)
		.replace("{history}", &history)
		.replace("{execution_ledger}", &execution_ledger);

	eprintln!(
		"{}",
		format_kv("FINAL PROMPT CHARS:", &prompt.len().to_string())
	);
	eprintln!(
		"{}",
		format_kv("FINAL PROMPT LINES:", &prompt.lines().count().to_string())
	);
	eprintln!("{}", format_kv("build_prompt:", "⛔️ RETURN"));

	prompt
}
pub fn build_prompt_from_ctx(ctx: &AgentCtx) -> String {
	let workspace = format_workspace(&ctx.workspace);
	let history = format_history(&ctx.history, 3, 3);
	let prompt = build_sys_action(
		ACTION_PROMPT_EXECUTION,
		&[&ctx.prompt.as_deref().unwrap_or(""), &workspace, &history],
	);
	section!("build_prompt_from_ctx");
	println!("ctx.history ({} entries):", ctx.history.len());
	println!(
		"prompt ({} lines, {} chars):\n{}",
		prompt.lines().count(),
		prompt.len(),
		preview_lines(&prompt, PROMPT_PREVIEW_LINES)
	);
	prompt
}
pub fn build_sys_action(template: &str, args: &[&str]) -> String {
	let placeholders = ["{task}", "{workspace}", "{history}"];

	debug_assert_eq!(
		placeholders.len(),
		args.len(),
		"build_sys_action: placeholder count doesn't match argument count",
	);
	let mut prompt = template.to_string();
	for (placeholder, arg) in placeholders.iter().zip(args) {
		prompt = prompt.replace(placeholder, arg);
	}
	prompt
}
pub fn build_sys_prompt(template: &str, prompt: &str) -> String {
	template.replace("{}", prompt)
}

pub fn language_from_extension(extension: &str) -> Option<String> {
	let language = match extension {
		"rs" => "Rust",
		"js" => "JavaScript",
		"jsx" => "JavaScript",
		"ts" => "TypeScript",
		"tsx" => "TypeScript",
		"py" => "Python",
		"go" => "Go",
		"java" => "Java",
		"c" => "C",
		"h" => "C",
		"cpp" => "C++",
		"cc" => "C++",
		"cxx" => "C++",
		"hpp" => "C++",
		"cs" => "C#",
		"swift" => "Swift",
		"kt" => "Kotlin",
		"kts" => "Kotlin",
		"rb" => "Ruby",
		"php" => "PHP",
		"sh" => "Shell",
		"bash" => "Shell",
		"zsh" => "Shell",
		"fish" => "Shell",
		"html" => "HTML",
		"css" => "CSS",
		"scss" => "SCSS",
		"json" => "JSON",
		"toml" => "TOML",
		"yaml" => "YAML",
		"yml" => "YAML",
		"xml" => "XML",
		"md" => "Markdown",
		"sql" => "SQL",
		_ => return None,
	};
	Some(language.to_string())
}
pub async fn ollama_generate(prompt: &str, system: Option<&str>, json: bool) -> Result<String> {
	let client = reqwest::Client::new();

	let mut payload = serde_json::json!({
		"model": DEFAULT_MODEL,
		"prompt": prompt,
		"stream": false,
	});

	if let Some(sys_msg) = system {
		payload["system"] = serde_json::json!(sys_msg);
	}

	if json {
		payload["format"] = serde_json::json!("json");
	}

	let response = client
		.post(AGENT_GEN_URL)
		.json(&payload)
		.send()
		.await
		.context("Failed to send Ollama request")?;

	let status = response.status();
	let body = response
		.text()
		.await
		.context("Failed to read Ollama response body")?;

	if !status.is_success() {
		anyhow::bail!(
			"Ollama returned HTTP {status}: {}",
			body.chars().take(2_000).collect::<String>()
		);
	}

	let res: serde_json::Value =
		serde_json::from_str(&body).context("Ollama returned invalid response JSON")?;

	if let Some(error) = res.get("error").and_then(|v| v.as_str()) {
		anyhow::bail!("Ollama error: {error}");
	}

	res
		.get("response")
		.and_then(serde_json::Value::as_str)
		.map(str::to_owned)
		.ok_or_else(|| {
			anyhow::anyhow!(
				"Ollama response missing string field `response`: {}",
				body.chars().take(2_000).collect::<String>()
			)
		})
}
pub fn preview_lines(text: &str, max_lines: usize) -> String {
	let lines: Vec<&str> = text.lines().collect();
	let (lines, truncated) = if lines.len() <= max_lines {
		(lines.as_slice(), 0)
	} else {
		(&lines[..max_lines], lines.len() - max_lines)
	};
	let mut output = String::new();
	for line in lines {
		output.push_str("\x1b[2m  ");
		output.push_str(line);
		output.push_str("\x1b[0m\n");
	}
	if truncated > 0 {
		output.push_str("\n");
		output.push_str(&format!(
			"\x1b[2m  ... {} more lines truncated\x1b[0m\n",
			truncated
		));
	}
	output
}
pub fn preview(value: impl std::fmt::Debug, max_len: usize) -> String {
	let value = format!("{value:?}");
	if value.len() > max_len {
		format!(
			"{}... [truncated, true_len={}]",
			&value[..max_len],
			value.len()
		)
	} else {
		value
	}
}
pub async fn prompt_chat(ctx: &AgentCtx) -> Result<String> {
	let prompt = structured_prompt_chat(ctx);
	let result = ollama_generate(&prompt, None, false).await?;
	Ok(result)
}
pub async fn prompt_ollama_json<T>(prompt: &str) -> Result<T>
where
	T: DeserializeOwned,
{
	let raw = ollama_generate(prompt, Some("You are a helpful assistant"), true)
		.await
		.context("Ollama generation failed")?;
	serde_json::from_str(&raw).with_context(|| {
		format!(
			"Failed to deserialize Ollama JSON response ({} bytes): {:?}",
			raw.len(),
			raw.chars().take(2_000).collect::<String>(),
		)
	})
}
pub fn structured_prompt_chat(ctx: &AgentCtx) -> String {
	format!(
		r#"
			You are a helpful assistant.
			User request:
			{}
			History:
			{}
			Respond normally. No JSON. Just text.
		"#,
		&ctx.prompt.as_deref().unwrap_or(""),
		format_history(&ctx.history, 3, 3)
	)
}

#[derive(Clone, Debug)]
pub struct ExecutionRecord {
	pub step: usize,
	pub action: String,
	pub cwd: Option<PathBuf>,
	pub outcome: ExecutionOutcome,
	pub exit_code: Option<i32>,
	pub stdout: String,
	pub stderr: String,
}

#[derive(Clone, Debug)]
pub enum ExecutionOutcome {
	Succeeded,
	Failed,
	Rejected { reason: String },
}
