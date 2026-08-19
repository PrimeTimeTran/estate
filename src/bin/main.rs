//! Estate Engine CLI.
//!
//! Install:
//! ```sh
//! cargo install --path . --bin estate
//! ```
//!
//! Usage:
//! ```sh
//! estate fmt path/to/file.rs
//! ```

use ::estate::{prelude::*, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let parsed_cli = cli::parse();

	match parsed_cli.command {
		cli::Command::Tray => {
			let engine = EstateEngine::new()?;
			app::App::run_tray_daemon(engine)?;
		}

		cli::Command::Daemon { live: true } => {
			let engine = EstateEngine::new()?;
			let mut daemon = BackgroundDaemon::new(engine);

			daemon.start(DaemonOptions { foreground: true }).await?;
		}

		cli::Command::Daemon { live: false } => {
			App::spawn_tray_process()?;
		}

		_ => {
			let cli_ctx = cli::Context::new();
			let engine = EstateEngine::new()?;

			router::execute(parsed_cli, cli_ctx, engine).await?;
		}
	}

	Ok(())
}
