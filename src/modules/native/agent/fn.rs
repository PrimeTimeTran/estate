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
