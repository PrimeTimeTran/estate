/// # Estate Engine CLI
///
/// Build and install the `estate` CLI.
///
/// ## Development Build
///
/// ```sh
/// cargo build --bin estate
/// ```
///
/// Run the development binary:
///
/// ```sh
/// ./target/debug/estate fmt path/to/file.rs
/// ./target/debug/estate format path/to/file.rs
/// ```
///
/// ## Release Build
///
/// ```sh
/// cargo build --release --bin estate
/// ```
///
/// Run the release binary:
///
/// ```sh
/// ./target/release/estate fmt path/to/file.rs
/// ```
///
/// ## Install
///
/// Install the CLI into Cargo's binary directory:
///
/// ```sh
/// cargo install --path . --bin estate
/// ```
///
/// Once installed:
///
/// ```sh
/// estate fmt path/to/file.rs
/// estate format path/to/file.rs
/// ```
use crate::{
	Runtime,
	app::{model, *},
	doc,
	native::daemon::projection::command,
};

pub(crate) async fn execute<R: Runtime>(
	parsed_cli: Cli,
	ctx: cli::context::Context,
	engine: model::EstateEngine<R>,
) -> Result<(), Error> {
	let command = parsed_cli.command.unwrap_or(Command::Start { tail: false });
	match command {
		Command::Start { tail: false } => {
			// app::App::spawn_tray_process();
		}
		Command::Tray => {
			// App::start_app(engine)?;
		}
		Command::Format(args) => {
			engine.format(&args).await;
		}
		Command::Metrics(args) => {
			let workspace = AnalyzeDaemon.run(&ctx, &args).await?;
			AnalyzeLoop::run_cli(workspace).await;
		}
		Command::Analyze(args) => {
			doc!(
				r#"
				          ┌──────────────────┐
				          │      Estate      │
				          │  core services   │
				          └────────┬─────────┘
				                  │
				    ┌─────────────┼─────────────┐
				    │             │             │
				    ▼             ▼             ▼
				CLI command     Daemon         LSP
				(short-lived)   (long-lived)   (long-lived)
				                  │
				                  │
				            ┌─────┴─────┐
				            │           │
				            ▼           ▼
				          Headless     Menu Bar
				          process      application
			"#
			);
			let mut stream = match UnixStream::connect(SOCKET_PATH).await {
				Ok(s) => s,
				Err(e) => {
					return Err(anyhow::anyhow!(
						"Daemon is not running. Start it first: {e}"
					));
				}
			};
			for path in &args.paths {
				let clean_path = path.canonicalize().unwrap_or_else(|_| path.clone());
				let request = serde_json::json!({
						"path": clean_path,
						"line": args.line,
						"column": &args.column,
						"mode": args.mode
				});
				let payload = format!("{}\n", request);
				if let Err(e) = stream.write_all(payload.as_bytes()).await {
					eprintln!("Failed to send request: {}", e);
					break;
				}
				let mut buf = Vec::new();
				match stream.read_to_end(&mut buf).await {
					Ok(_) => {
						let response = String::from_utf8_lossy(&buf);
						print!("{}", response);
					}
					Err(e) => {
						eprintln!("Failed to read response: {}", e);
					}
				}
			}
		}
		Command::DaemonServer => {
			native::daemon::DaemonServer::run().await;
		}
		Command::Capabilities(args) => match AnalyzeDaemon.run(&ctx, &args).await {
			Ok(result) => {
				MetricsRenderer::render(&result);
			}
			Err(err) => {
				eprintln!("Analyze failed: {err}");
				std::process::exit(1);
			}
		},
		Command::Status => StatusDaemon.run(&ctx).await,
		Command::Bookmark => command::ViewList.run(&ctx).await,
		Command::Bookmarks => command::ViewList.run(&ctx).await,
		Command::Explain => command::Explain.run(&ctx).await,
		Command::ExplainDoc => command::ExplainDoc.run(&ctx).await,
		Command::View { name } => (command::View { name }).run(&ctx).await,
		Command::ViewFork { name } => (command::ViewFork { name }).run(&ctx).await,
		Command::ViewList => command::ViewList.run(&ctx).await,
		Command::Deps { name } => (command::Deps { name }).run(&ctx).await,
		Command::Foo(_args) => {
			let _ = EstateDiscovery::init();
		}
		_ => {
			todo!("Command not found");
		}
	}
	Ok(())
}
