use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::UnixStream,
};

use crate::{
	_core::EstateDiscovery,
	constants::*,
	daemon::{
		daemon::*,
		projection::command::*,
		start::{BackgroundDaemon, Daemon, DaemonOptions},
		*,
	},
};
use cli::{CliCommand, Command, Context as CliContext};
use revelation::analyzer::Workspace;

pub async fn execute(parsed_cli: cli::Cli, ctx: CliContext, _est_cxt: app::Context) {
	match parsed_cli.command {
		// 1. The Daemon Server Command (bootstraps your background server)
		// - live and watch requests come in
		// $ cg-rb loi daemon --live
		// - normally
		// cg-rb loi daemon
		Command::Daemon { live } => {
			let workspace = Workspace::new();
			let ctx = app::Context::new(app::ContextSource::Cli).expect("failed creating estate context");
			let mut daemon = BackgroundDaemon::new(workspace, ctx);
			// OS Menubar
			// daemon.run().await?;
			let options = DaemonOptions { foreground: live };
			if let Err(e) = daemon.start(options).await {
				eprintln!("Daemon error: {}", e);
			}
		}
		// 2. The Analyze Client Command (pings the socket and prints response)
		// Inside your Command::Analyze handler:
		Command::Analyze(args) => {
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
				let request_payload = serde_json::json!({
						"path": clean_path,
						"line": args.line,
						"column": &args.column,
						"mode": args.mode
				});
				let payload = format!("{}\n", request_payload);
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
		Command::Foo(_args) => {
			let _ = EstateDiscovery::init();
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

		// Command::Analyze(args) =>  {
		//     let workspace = Workspace::new();
		//     // 1. Initialize the daemon and get the message sender handle
		//     let (daemon, tx) = start::BackgroundDaemon::new(workspace);

		//     // 2. Spawn the daemon into the background so it doesn't block the CLI
		//     tokio::spawn(async move {
		//         daemon.run().await;
		//     });

		//     // 3. Send an initial request to trigger it immediately (if desired)
		//     if let Err(e) = tx.send(AnalyzeRequest::RunAnalysis).await {
		//         eprintln!("Failed to send analysis request to background daemon: {}", e);
		//     }

		//     // start::BackgroundDaemon::run(&ctx, &args).await;
		//     // let workspace = Workspace::new();
		//     // AnalyzeLoop::run();
		//     // Ok(workspace) => {
		//     // }
		//     // Err(err) => {
		//     //     eprintln!("Analyze failed: {err}");
		//     //     std::process::exit(1);
		//     // }
		// },
		// Command::Analyze(args) => match AnalyzeDaemon.run(&ctx, &args).await {
		//     Ok(workspace) => {
		//         AnalyzeLoop::run(workspace).await;
		//     }
		//     Err(err) => {
		//         eprintln!("Analyze failed: {err}");
		//         std::process::exit(1);
		//     }
		// },
		Command::Format(args) => {
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
		Command::Stop => StopDaemon.run(&ctx).await,
		Command::Bookmark => ViewList.run(&ctx).await,
		Command::Bookmarks => ViewList.run(&ctx).await,
		Command::Reload => ReloadDaemon.run(&ctx).await,
		Command::Explain => Explain.run(&ctx).await,
		Command::ExplainDoc => ExplainDoc.run(&ctx).await,
		Command::View { name } => View { name }.run(&ctx).await,
		Command::ViewFork { name } => ViewFork { name }.run(&ctx).await,
		Command::ViewList => ViewList.run(&ctx).await,
		Command::Deps { name } => Deps { name }.run(&ctx).await,
		_ => {
			todo!("")
		}
	}
}
