use crate::prelude::{daemon::projection::command, estate, *};
use cli::prelude::{Command, *};
use revelation::analyzer::Workspace;

pub async fn execute(parsed_cli: cli::Cli, ctx: cli::Context, app: app::Context) {
	match parsed_cli.command {
		Command::Foo(_args) => {
			let _ = EstateDiscovery::init();
		}
		Command::Format(args) => {
			LintDaemon.run(&ctx, &args).await;
		}
		Command::Metrics(args) => match AnalyzeDaemon.run(&ctx, &args).await {
			Ok(workspace) => {
				// "/Users/future/KB/project/crates/estate-engine/src/main.rs"
				// $ cargo run metrics "/Users/future/KB/project/crates/estate-engine/src/main.rs"
				AnalyzeLoop::run_cli(workspace).await;
			}
			Err(err) => {
				eprintln!("Analyze failed: {err}");
				std::process::exit(1);
			}
		},
		Command::Daemon { live } => {
			let workspace = Workspace::new();
			let ctx = app::Context::new(app::ContextSource::Cli).expect("failed creating estate context");
			// OS Menubar
			// daemon.run().await?;
			let mut daemon = BackgroundDaemon::new(workspace, ctx);
			let options = DaemonOptions { foreground: live };
			if let Err(e) = daemon.start(options).await {
				eprintln!("Daemon error: {}", e);
			}
		}
		Command::Analyze(args) => {
			eprint!("analyze");
			let mut stream = match UnixStream::connect(SOCKET_PATH).await {
				Ok(s) => s,
				Err(_) => {
					eprintln!("Daemon is not running! Start it first.");
					return;
				}
			};
			for path in &args.paths {
				let clean_path = path.canonicalize().unwrap_or_else(|_| path.clone());
				// Just pass the file path and line/offset directly as raw fields
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
				// Read response from socket server
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
		Command::Bar(args) => {
			LintDaemon.run(&ctx, &args).await;
		}
		Command::Spam(args) => {
			LintDaemon.run(&ctx, &args).await;
		}
		Command::Ham(args) => {
			LintDaemon.run(&ctx, &args).await;
		}
		// Server
		Command::Start => {
			start::daemon().await;
		}
		// Background process/app
		// Command::Process(args) => {
		//     start::BackgroundDaemon::run(&ctx, &args).await;
		// }
		Command::DaemonServer => {
			start::DaemonServer::run().await;
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
		Command::Stop => stop::StopDaemon.run(&ctx).await,
		Command::Bookmark => command::ViewList.run(&ctx).await,
		Command::Bookmarks => command::ViewList.run(&ctx).await,
		Command::Reload => reload::ReloadDaemon.run(&ctx).await,
		Command::Explain => command::Explain.run(&ctx).await,
		Command::ExplainDoc => command::ExplainDoc.run(&ctx).await,
		Command::View { name } => command::View { name }.run(&ctx).await,
		Command::ViewFork { name } => command::ViewFork { name }.run(&ctx).await,
		Command::ViewList => command::ViewList.run(&ctx).await,
		Command::Deps { name } => command::Deps { name }.run(&ctx).await,
		_ => {
			todo!("")
		}
	}
}
