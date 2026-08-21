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

use ::estate::{logger, prelude::*, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let parsed_cli = cli::context::parse();
	// let log_config = logger::LogConfig::load(&parsed_cli)?;
	// logger::init_logging(&log_config)?;
	// tracing::info!(target: "estate", "starting Estate");

	let mut config = LogConfig::load()?;
	config.apply_cli(&parsed_cli)?;
	logger::init_logging(&config)?;

	let engine = EstateEngine::new()?;

	match parsed_cli.command {
		cli::context::Command::Tray => {
			App::run_tray_daemon(engine)?;
		}

		cli::context::Command::Start { tail: true } => {
			tracing::info!(target: "estate", "spawning tray");

			App::spawn_tray_process().await?;

			tracing::info!(target: "estate", "starting foreground daemon");

			let mut daemon = Daemon::new(engine);

			daemon.start(DaemonOptions { foreground: true }).await?;
		}

		cli::context::Command::Start { tail: false } => {
			App::spawn_tray_process().await?;
		}

		_ => {
			let cli_ctx = cli::context::Context::new();
			router::execute(parsed_cli, cli_ctx, engine).await?;
		}
	}

	Ok(())
}
