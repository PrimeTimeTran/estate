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
pub fn build_prompt(ctx: &AgentContext) -> String {
	let workspace = format_workspace(&ctx.workspace);
	let history = format_history(&ctx.history);
	let prompt = build_sys_action(
		ACTION_PROMPT_EXECUTION,
		&[&ctx.prompt, &workspace, &history],
	);
	section!("BUILT PROMPT");
	println!(
		"prompt ({} lines, {} chars):\n{}",
		prompt.lines().count(),
		prompt.len(),
		preview_lines(&prompt, PROMPT_PREVIEW_LINES)
	);
	prompt
}
pub fn build_sys_action(template: &str, args: &[&str]) -> String {
	let mut prompt = template.to_string();
	for arg in args {
		if let Some((before, after)) = prompt.split_once("{}") {
			prompt = format!("{before}{arg}{after}");
		}
	}
	prompt
}
pub fn build_sys_prompt(template: &str, prompt: &str) -> String {
	template.replace("{}", prompt)
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

pub async fn prompt_chat(ctx: &AgentContext) -> Result<String> {
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

pub fn structured_prompt_chat(ctx: &AgentContext) -> String {
	format!(
		r#"
			You are a helpful assistant.
			User request:
			{}
			History:
			{}
			Respond normally. No JSON. Just text.
		"#,
		ctx.prompt,
		format_history(&ctx.history)
	)
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
