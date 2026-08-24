//! Estate Engine CLI.
//!
//! Install:
//! ```sh
//! cargo install --path . --bin estate
//! ```
//!
//! Usage:
//! Start Estate Engine:
//! ```sh
//! estate start
//! ```
//!
//! ```sh
//! estate fmt path/to/file.rs
//! ```
use estate::prelude::{logger, *};

fn main() -> anyhow::Result<()> {
	let cli = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;
	let trace = Tracer::new("app");
	let mut flow = trace.flow("init");
	flow.info("App::new");
	let mut app = App::new()?;
	flow.info("App::run");
	let result = app.run(cli);
	flow.info("Main exit");
	result
}
