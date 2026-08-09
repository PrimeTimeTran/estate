mod _core;
mod daemon;
mod estate;
mod vfs;
use crate::daemon::{
	app::{self, run_tray_daemon},
	router,
};
use cli::Command;

// #[tokio::main]
// async fn main2() {
// 	let estate_ctx =
// 		app::Context::new(app::ContextSource::Cli).expect("failed creating estate context");
// 	let parsed_cli = cli::parse();
// 	let cli_ctx = cli::Context::new();
// 	router::execute(parsed_cli, cli_ctx, estate_ctx).await;
// }
#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let parsed_cli = cli::parse();
	match parsed_cli.command {
		Command::Daemon { .. } => {
			run_tray_daemon().await?;
		}
		_ => {
			let estate_ctx = app::Context::new(app::ContextSource::Cli)?;
			let cli_ctx = cli::Context::new();
			router::execute(parsed_cli, cli_ctx, estate_ctx).await;
		}
	}
	Ok(())
}
