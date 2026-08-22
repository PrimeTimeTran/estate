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

use ::estate::{logger, prelude::*, ve};

fn main() -> anyhow::Result<()> {
	println!("Hiiiii main");
	ve::start_global_scroll_daemon();
	let cli = cli::context::parse();

	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;

	let mut app = App::new()?;

	app.run(cli)
}
