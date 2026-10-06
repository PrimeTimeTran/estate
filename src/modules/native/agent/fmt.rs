use super::*;

pub fn format_workspace(workspace: &WorkspaceContext) -> String {
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
pub fn format_history(history: &[AgentObservation]) -> String {
	if history.is_empty() {
		return "No actions have been performed yet.".into();
	}
	let mut output = String::new();
	for (index, observation) in history.iter().enumerate() {
		output.push_str(&format!("{}. {:?}\n", index + 1, observation));
	}
	output
}
