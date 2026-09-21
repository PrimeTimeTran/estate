use demand::{Confirm, DemandOption, Dialog, DialogButton, Input, MultiSelect, Select, Spinner};

use anyhow::Result;

use estate::modules::wizard;

pub fn main() -> Result<()> {
	println!();
	println!("Estate CLI playground");
	println!();

	// ─────────────────────────────────────────────
	// 1. INPUT
	// - Plain text input
	// - Can have suggestions
	// ─────────────────────────────────────────────

	let name = Input::new("What's your name?")
		.description("We'll use this to personalize your experience.")
		.placeholder("Enter your name")
		.suggestions(&[
			"Adam Grant",
			"Danielle Steel",
			"Eveline Widmer-Schlumpf",
			"Robert De Niro",
			"Sarah Michelle Gellar",
			"Zack Snyder",
		])
		.validation(|s: &str| {
			if s.is_empty() {
				return Err("Name cannot be empty");
			}

			if s.len() < 5 {
				return Err("Name must be at least 5 characters");
			}

			Ok(())
		})
		.run()
		.expect("input failed");

	// println!("Hello, {name}!");

	// ─────────────────────────────────────────────
	// 2. SELECT
	// ─────────────────────────────────────────────

	let project_type = Select::new("What are you building?")
		.description("[Select] Choose the primary type of project.")
		.filterable(true)
		.option(DemandOption::new("Rust Application").description("A native Rust application"))
		.option(DemandOption::new("CLI").description("A command-line application"))
		.option(DemandOption::new("LSP").description("A language server"))
		.option(DemandOption::new("Daemon").description("A long-running background service"))
		.option(DemandOption::new("Web").description("A WebAssembly application"))
		.run()
		.expect("select failed");

	println!("Project type: {project_type}");

	// ─────────────────────────────────────────────
	// 3. MULTISELECT
	// ─────────────────────────────────────────────

	let features = MultiSelect::new("Which features do you want?")
		.description("[Multi Select] You can select multiple features.")
		.filterable(true)
		.option(DemandOption::new("CLI").selected(true))
		.option(DemandOption::new("LSP"))
		.option(DemandOption::new("Daemon"))
		.option(DemandOption::new("GUI"))
		.option(DemandOption::new("Web"))
		.option(DemandOption::new("MCP"))
		.min(1)
		.max(6)
		.run()
		.expect("multiselect failed");

	println!("Features: {features:?}");

	// ─────────────────────────────────────────────
	// 4. CONFIRM
	// ─────────────────────────────────────────────

	let initialize = Confirm::new("Initialize the project?")
		.description("This will create the project structure.")
		.affirmative("Yes, initialize")
		.negative("No, cancel")
		.run()
		.expect("confirm failed");

	println!("Initialize: {initialize}");

	// ─────────────────────────────────────────────
	// 5. DIALOG
	// ─────────────────────────────────────────────

	let action = Dialog::new("What should we do next?")
		.description("Choose an action.")
		.buttons(vec![
			DialogButton::new("Build"),
			DialogButton::new("Run"),
			DialogButton::new("Test"),
			DialogButton::new("Exit"),
		])
		.run()
		.expect("dialog failed");

	println!("Action: {action}");

	// ─────────────────────────────────────────────
	// 6. LIST
	// ─────────────────────────────────────────────

	let command = Select::new("Choose a command")
		.option(DemandOption::new("Build").description("Compile the project"))
		.option(DemandOption::new("Test").description("Run the test suite"))
		.option(DemandOption::new("Run").description("Run the application"))
		.option(DemandOption::new("Clean").description("Remove build artifacts"));

	let command = command.run()?;

	println!("Command: {}", command);

	// ─────────────────────────────────────────────
	// 7. SPINNER
	// ─────────────────────────────────────────────

	Spinner::new("Building...")
		.run(|_| {
			// do the work
		})
		.expect("error running spinner");

	println!("Build complete");

	// ─────────────────────────────────────────────
	// 8. SUMMARY
	// ─────────────────────────────────────────────

	println!("──────────────────────────────────────────────");
	println!("Project summary");
	println!("──────────────────────────────────────────────");
	println!("Name:     {name}");
	println!("Type:     {project_type}");
	println!("Features: {features:?}");
	println!("──────────────────────────────────────────────");
	Ok(())
}
