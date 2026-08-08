mod _core;
mod daemon;
mod estate;
mod vfs;

use crate::daemon::{app, router};

#[tokio::main]

async fn main() {
	let estate_ctx =
		app::Context::new(app::ContextSource::Cli).expect("failed creating estate context");
	let parsed_cli = cli::parse();
	let cli_ctx = cli::Context::new();
	router::execute(parsed_cli, cli_ctx, estate_ctx).await;
}
