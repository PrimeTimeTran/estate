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
//!
//! ```sh
//!
//! ```
//!
//! ```sh
//!
//! ```
//!
//! ```sh
//!
//! ```
//!
//! ```sh
//!
//! ```
//!
//! ```sh
//!
//! ```
//!

use ::estate::{prelude::*, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let parsed_cli = cli::context::parse();
	let engine = EstateEngine::new()?;
	match parsed_cli.command {
		cli::context::Command::Tray => {
			App::run_tray_daemon(engine)?;
		}
		cli::context::Command::Start { tail: true } => {
			App::spawn_tray_process().await?;
			let mut daemon = BackgroundDaemon::new(engine);
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
