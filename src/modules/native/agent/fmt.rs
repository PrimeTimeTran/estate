use super::*;

pub fn format_workspace(workspace: &CtxWorkspace) -> String {
	let mut output = String::new();
	output.push_str(&format!("CWD: {}\n", workspace.cwd.display()));
	if workspace.files.is_empty() {
		output.push_str("FILES: none discovered\n");
	} else {
		output.push_str("FILES:\n");
		for file in &workspace.files {
			output.push_str(&format!("- {}\n", file.path));
		}
	}
	output
}
pub fn format_history<T: std::fmt::Debug>(
	history: &[T],
	max_entries: usize,
	max_lines_per_entry: usize,
) -> String {
	let mut output = format!(
		"{} ({})\n",
		colorize(CYAN, "HISTORY"),
		history.len(),
	);

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
pub fn format_preview(
	label: &str,
	content: &str,
	max_lines: usize,
) -> String {
	let mut output = format!("{}:\n", colorize(BOLD, label));

	for line in content.lines().take(max_lines) {
		output.push_str("  ");
		output.push_str(line);
		output.push('\n');
	}

	let total_lines = content.lines().count();

	if total_lines > max_lines {
		output.push_str(&format!(
			"  {} ({} more lines)\n",
			colorize(DIM, "..."),
			total_lines - max_lines,
		));
	}

	output
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
		colorize(if errors > 0 { RED } else { DIM }, format!("errors={errors}")),
		colorize(YELLOW, format!("logs={logs}")),
		colorize(MAGENTA, format!("tokens={tokens}")),
		colorize(CYAN, format!("files={files}")),
	)
}