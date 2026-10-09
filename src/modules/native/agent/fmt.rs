use super::*;
use std::fmt::Write;

pub fn format_kv(k: &str, v: &str) -> String {
	format!("{} {}", colorize(BOLD, k), colorize(DIM, v),)
}

pub fn format_execution_ledger(
	commands: &[CommandRecord],
	max_records: usize,
	max_output_lines: usize,
	max_output_chars: usize,
) -> String {
	if commands.is_empty() {
		return colorize(DIM, "No commands have been executed yet.");
	}

	let start = commands.len().saturating_sub(max_records);
	let recent = &commands[start..];
	let mut output = String::new();

	let _ = writeln!(
		output,
		"{} {}",
		colorize(BOLD, "COMMANDS:"),
		colorize(
			DIM,
			format!(
				"{} record(s); showing the most recent {}.",
				commands.len(),
				recent.len(),
			),
		),
	);

	for (index, command) in recent.iter().enumerate() {
		let _ = writeln!(
			output,
			"\n{} {}",
			colorize(BOLD, &format!("COMMAND {}", start + index + 1)),
			colorize(DIM, &command.command),
		);
		if let Some(code) = command.exit_code {
			let _ = writeln!(
				output,
				"{} {}",
				colorize(BOLD, "Exit code:"),
				colorize(DIM, code.to_string()),
			);
		}

		for (label, content) in [
			("stdout:", command.stdout.as_str()),
			("stderr:", command.stderr.as_str()),
		] {
			if !content.is_empty() {
				let _ = writeln!(output, "{}", colorize(BOLD, label));
				let truncated = truncate_output(content, max_output_chars);
				let preview = preview_lines(&truncated, max_output_lines);
				let _ = writeln!(output, "{}", colorize(DIM, preview));
			}
		}
	}

	output
}

pub fn format_workspace(workspace: &CtxWorkspace) -> String {
	let mut output = format!(
		"{} {}\n",
		colorize(BOLD, "CWD:"),
		colorize(DIM, workspace.cwd.display().to_string()),
	);
	if workspace.files.is_empty() {
		output.push_str(&format!(
			"{} {}\n",
			colorize(BOLD, "FILES:"),
			colorize(DIM, "none discovered"),
		));
	} else {
		output.push_str(&format!(
			"{} {}\n",
			colorize(BOLD, "FILES:"),
			colorize(DIM, format!("{} discovered", workspace.files.len())),
		));
		for file in &workspace.files {
			output.push_str(&format!(
				"  {} {}\n",
				colorize(DIM, "•"),
				colorize(DIM, &file.path),
			));
		}
	}
	output
}
pub fn format_history<T: std::fmt::Debug>(
	history: &[T],
	max_entries: usize,
	max_lines_per_entry: usize,
) -> String {
	let mut output = format!("{} ({})\n", colorize(CYAN, "HISTORY"), history.len(),);

	for (index, entry) in history.iter().take(max_entries).enumerate() {
		let rendered = format!("{entry:#?}");
		output.push_str(&format!(
			"{} {}\n",
			colorize(DIM, format!("[{index}]")),
			format_preview("entry", &rendered, max_lines_per_entry),
		));
	}

	if history.len() > max_entries {
		output.push_str(&format!(
			"{} more entries omitted\n",
			history.len() - max_entries,
		));
	}

	output
}
pub fn format_preview(label: &str, content: &str, max_lines: usize) -> String {
	format!(
		"{}:\n{}",
		colorize(BOLD, label),
		preview_lines(content, max_lines),
	)
}
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const MAGENTA: &str = "\x1b[35m";
const BOLD: &str = "\x1b[1m";

pub fn colorize(color: &str, value: impl std::fmt::Display) -> String {
	format!("{color}{value}{RESET}")
}

pub fn format_agent_metrics(
	history: usize,
	commands: usize,
	errors: usize,
	logs: usize,
	tokens: usize,
	files: usize,
) -> String {
	format!(
		"{} {} {} {} {} {}",
		colorize(CYAN, format!("history={history}")),
		colorize(GREEN, format!("cmds={commands}")),
		colorize(
			if errors > 0 { RED } else { DIM },
			format!("errors={errors}")
		),
		colorize(YELLOW, format!("logs={logs}")),
		colorize(MAGENTA, format!("tokens={tokens}")),
		colorize(CYAN, format!("files={files}")),
	)
}

pub fn truncate_output(value: &str, max_chars: usize) -> String {
	let mut chars = value.chars();
	let truncated: String = chars.by_ref().take(max_chars).collect();

	if chars.next().is_some() {
		format!("{truncated}\n[Output truncated; additional output omitted]")
	} else {
		truncated
	}
}
