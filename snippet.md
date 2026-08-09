```rust
pub async fn execute(parsed_cli: cli::Cli, ctx: CliContext, _est_cxt: app::Context) {
	// let pi = 3.14;
	// let result = make_shape(pi);
	// fn make_shape(number: f64) -> f64 {
	//     return number;
	// }
	// let doubled = result + result;
	match parsed_cli.command {
		// 1. The Daemon Server Command (bootstraps your background server)
		// - live and watch requests come in
		// $ cg-rb loi daemon --live
		// - normally
		// cg-rb loi daemon
		Command::Daemon { live } => {
			let socket_path = "/tmp/loi_daemon.sock";
			let workspace = Workspace::new(); // Or build your workspace from est_cxt if needed
			let (daemon, _tx) = BackgroundDaemon::new(workspace);
			if let Err(e) = daemon.run_socket_server(socket_path, live).await {
				eprintln!("Daemon error: {}", e);
			}
		}
		// 2. The Analyze Client Command (pings the socket and prints response)
		// Inside your Command::Analyze handler:
		Command::Analyze(args) => {
			let socket_path = "/tmp/loi_daemon.sock";
			let mut stream = match UnixStream::connect(socket_path).await {
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
```
