use super::*;

pub async fn build_action(prompt: &str) -> Result<LlmAction> {
	let client = reqwest::Client::new();
	let system_prompt: &str = JSON_PROMPT_EXECUTION;
	let payload = serde_json::json!({
			"model": "qwen3:8b",
			"system": system_prompt,
			"prompt": prompt,
			"stream": false,
			"format": "json"
	});
	let res = client
		.post(crate::AGENT_GEN_URL)
		.json(&payload)
		.send()
		.await?
		.json::<serde_json::Value>()
		.await?;
	let response = res["response"].as_str().unwrap_or("");
	let action: serde_json::Value = serde_json::from_str(response)?;
	println!(
		"action choice   {}",
		action["message"].as_str().unwrap_or("")
	);
	let response_text = res["response"].as_str().unwrap_or("{}");
	let raw: LlmAction = serde_json::from_str(response_text)?;
	Ok(raw)
}
pub fn build_prompt(ctx: &AgentCtx) -> String {
	let workspace = format_workspace(&ctx.workspace);
	let history = format_history(&ctx.history);
	let prompt = ACTION_PROMPT_EXECUTION
		.replace("{task}", ctx.prompt.as_deref().unwrap_or(""))
		.replace("{workspace}", &workspace)
		.replace("{history}", &history);
	section!("build_prompt");
	println!(
		"prompt ({} lines, {} chars):\n{}",
		prompt.lines().count(),
		prompt.len(),
		preview_lines(&prompt, PROMPT_PREVIEW_LINES)
	);
	prompt
}
pub fn build_prompt_from_ctx(ctx: &AgentCtx) -> String {
	let workspace = format_workspace(&ctx.workspace);
	let history = format_history(&ctx.history);
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
			"model": "qwen3:8b",
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
		.post(crate::AGENT_GEN_URL)
		.json(&payload)
		.send()
		.await?;
	let res: serde_json::Value = response.json().await?;
	res["response"]
		.as_str()
		.map(|s| s.to_string())
		.ok_or_else(|| anyhow!("Failed to parse response field from Ollama"))
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
	let result = ollama_generate(prompt, Some("You are a helpful assistant"), true).await?;
	Ok(serde_json::from_str(&result)?)
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
		format_history(&ctx.history)
	)
}
