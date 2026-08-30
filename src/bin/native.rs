// Estate Engine CLI.

// Install:
// ```sh
// cargo install --path . --bin estate
// ```

// Usage:
// Start Estate Engine:
// ```sh
// estate start
// ```

// ```sh
// estate fmt path/to/file.rs
// ```

use estate::prelude::{logger, *};

fn main() -> Result<()> {
	let cli = cli::context::parse();

	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;

	let trace = Tracer::new("app");
	let flow = trace.flow("init");

	flow.info("App::new");

	let mut app = NativeApp::new()?;

	flow.info(">>> BEFORE app.run()");

	let result = app.run(cli);

	flow.info(">>> AFTER app.run(): {result}");
	flow.info(">>> main returning");

	std::process::exit(match result {
		Ok(()) => 0,
		Err(error) => {
			eprintln!("{error:#}");
			1
		}
	});
}
