use inquire::{Confirm, MultiSelect, Select, Text};

fn main() -> inquire::InquireResult<()> {
	println!();
	println!("  ┌─ Estate Setup ─────────────────────────────┐");
	println!("  │ Let's configure your project.              │");
	println!("  └─────────────────────────────────────────────┘");
	println!();

	// Text input
	let name = Text::new("Project name:")
		.with_default("my-project")
		.prompt()?;

	println!();

	// Single selection
	let kind = Select::new(
		"What kind of project is this?",
		vec![
			"Rust application",
			"Rust library",
			"Web application",
			"CLI tool",
		],
	)
	.with_help_message("↑/↓ to move • Enter to select")
	.prompt()?;

	println!();

	// Multiple selection
	let features = MultiSelect::new(
		"Which features do you want?",
		vec!["CLI", "LSP", "Daemon", "GUI", "Web", "MCP"],
	)
	.with_help_message("↑/↓ to move • Space to select • Enter to confirm")
	.prompt()?;

	println!();

	// Confirmation
	let initialize = Confirm::new("Initialize this project?")
		.with_default(true)
		.prompt()?;

	println!();
	println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
	println!("Project:   {name}");
	println!("Type:      {kind}");
	println!("Features:  {}", features.join(", "));
	println!("Initialize: {initialize}");
	println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

	Ok(())
}
