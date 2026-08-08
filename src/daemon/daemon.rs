use crate::daemon::*;
use cli::{CliCommand, Context as CliContext, FormatArgs};
use comfy_table::{Cell, Table, presets::UTF8_FULL};
use revelation::analyzer::*;
use serde::{Deserialize, Serialize};
use std::{
	fs,
	path::PathBuf,
	time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
///--------------------------------------------------------------------------------
/// Daemon
///--------------------------------------------------------------------------------
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DaemonState {
	pub longest_run: u64,
	pub starts: u64,
	#[serde(default)]
	pub status_checks: u64,
	pub started_at: u64,
}
impl DaemonState {
	pub fn save_workspace(path: &PathBuf) {
		println!("💾 save_workspace not implemented yet: {:?}", path);
	}
	pub fn now() -> u64 {
		SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs()
	}
}
impl DaemonState {
	fn path() -> std::io::Result<PathBuf> {
		Ok(engine_data_dir()?.join("state.json"))
	}
	pub fn load() -> Self {
		let path = Self::path().expect("could not resolve daemon state path");
		if !path.exists() {
			return Self {
				starts: 0,
				status_checks: 0,
				started_at: 0,
				longest_run: 0,
			};
		}
		let raw = fs::read_to_string(path).expect("failed reading daemon state");
		serde_json::from_str(&raw).expect("failed parsing daemon state")
	}
	pub fn save(state: &Self) {
		let path = Self::path().expect("could not resolve daemon state path");
		let json = serde_json::to_string_pretty(state).expect("failed serializing daemon state");
		fs::write(path, json).expect("failed writing daemon state");
	}
	pub fn record_status_check() {
		let mut state = Self::load();
		state.status_checks += 1;
		Self::save(&state);
	}
}
pub struct StatusDaemon;
#[async_trait::async_trait]
impl CliCommand for StatusDaemon {
	async fn run(&self, _ctx: &CliContext) {
		DaemonState::record_status_check();
		let state = DaemonState::load();
		let pid =
			std::fs::read_to_string("/tmp/estate-daemon.pid").unwrap_or_else(|_| "unknown".to_string());
		println!("📊 Estate Daemon Status");
		println!("──────────────────────");
		println!("✅ Status:          OK");
		println!("🆔 PID:             {}", pid);
		println!("🚀 Starts:          {}", state.starts);
		println!("🔎 Status checks:   {}", state.status_checks);
		println!("🕒 Started at:      {}", state.started_at);
		println!("⏱ Longest run:     {}s", state.longest_run);
		// 1. Server
		// match tokio::net::TcpStream::connect("127.0.0.1:7788").await {
		//     Ok(mut stream) => {
		//         tokio::io::AsyncWriteExt::write_all(&mut stream, b"status")
		//             .await
		//             .unwrap();
		//         let mut buf = vec![];
		//         tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut buf)
		//             .await
		//             .unwrap();
		//         println!("Daemon response:");
		//         println!("{}", String::from_utf8_lossy(&buf));
		//     }
		//     Err(err) => {
		//         println!("❌ Daemon socket unavailable: {}", err);
		//     }
		// }
		use tokio::io::{AsyncReadExt, AsyncWriteExt};
		// 1. Daemon
		match tokio::net::UnixStream::connect("/tmp/estate-daemon.sock").await {
			Ok(mut stream) => {
				stream.write_all(b"status\n").await.unwrap();
				let mut buf = vec![0; 1024];
				let n = stream.read(&mut buf).await.unwrap();
				println!("Daemon response:");
				println!("{}", String::from_utf8_lossy(&buf[..n]));
			}
			Err(err) => {
				println!("❌ Daemon socket unavailable: {}", err);
			}
		}
	}
}
pub struct LintDaemon;
impl LintDaemon {
	pub async fn run(&self, _ctx: &CliContext, args: &FormatArgs) {
		let compiler = DocCompiler::default();
		match compiler.run(&args.path) {
			Ok(_) => println!("Successfully formatted: {:?}", args.path),
			Err(e) => eprintln!("Error formatting file {:?}: {}", args.path, e),
		}
	}
}
pub struct AnalyzeDaemon;
impl AnalyzeDaemon {
	pub async fn run(
		&self,
		_ctx: &CliContext,
		args: &cli::FormatArgs,
	) -> Result<Workspace, AnalysisError> {
		let target_path = PathBuf::from(&args.path);
		let request = Analyze {
			target: AnalysisTarget::File(target_path.clone()),
			subject: None,
		};
		let analyzer = RustAnalyzer;

		let options = AnalyzerOptions::default();
		let workspace = analyzer.analyze(request, &options)?;
		let _metrics = workspace.metrics();
		Ok(workspace)
	}
}
// Define the type of request/message your loop handles
pub enum AnalysisRequest {
	AnalyzeWorkspace,
	// Add other request types here if needed
}
pub struct AnalyzeLoop {
	rx: mpsc::Receiver<AnalysisRequest>,
	workspace: Workspace,
}
impl AnalyzeLoop {
	// [Request] handler
	pub async fn run(mut self) {
		let actions = ActionRegistry::from_analysis(&self.workspace);

		// This loop pauses completely until a message is sent over the channel
		while let Some(request) = self.rx.recv().await {
			match request {
				AnalysisRequest::AnalyzeWorkspace => {
					println!("Processing workspace request & building AST...");
					let _analyze_action = actions.iter().find(|a| a.title == "analyze.workspace");
				}
			}
		}
	}
	// [CLI] handler
	pub async fn run_cli(workspace: Workspace) {
		loop {
			Self::render_context(&workspace);
			let actions = ActionRegistry::from_analysis(&workspace);
			let options = actions
				.iter()
				.map(|action| demand::DemandOption::new(action.title.clone()))
				.collect::<Vec<_>>();
			let choice = demand::Select::new("What would you like to do?")
				.options(options)
				.run();
			match choice {
				Ok(selected) => {
					if let Some(action) = actions.iter().find(|a| a.title == selected) {
						action.execute(&workspace);
					}
				}
				Err(_) => break,
			}
		}
	}
	fn render_context(result: &Workspace) {
		println!();
		println!("Workspace");
		println!("--------------------------------");
		let metrics = &result.metrics();
		println!("{:<20} {}", "Files", metrics.files.len());
		println!("{:<20} {}", "Packages", metrics.packages.len());
		println!("{:<20} {}", "Modules", metrics.modules.len());
		// println!("{:<20} {}", "Types", metrics.types.len());
		// println!("{:<20} {}", "Symbols", metrics.symbols.len());
		// println!("{:<20} {}", "Functions", metrics.functions.len());
		println!();
	}
}
// impl AnalyzeLoop {
//     pub async fn run(workspace: Workspace) {
//         let actions = ActionRegistry::from_analysis(&workspace);

//         loop {
//             // 1. Wait for an event, or check for changes
//             // (Placeholder: replace this with your file watch receiver or debounce logic)
//             println!("Waiting for workspace changes...");

//             // Example: simple polling interval with a guard, or an event listener .await
//             sleep(Duration::from_secs(1)).await;

//             // 2. Process workspace / build AST only when triggered
//             if let Some(action) = actions.iter().find(|a| a.title == "analyze.workspace") {
//                 println!("Running workspace analysis...");
//                 // Build AST here...
//             }
//         }
//     }
// }
///--------------------------------------------------------------------------------
/// CLI Tools
///--------------------------------------------------------------------------------
pub struct Action {
	pub id: String,
	pub title: String,
	pub description: String,
	pub enabled: bool,
	pub reason: Option<String>,
	pub category: ActionCategory,
	pub handler: Option<Handler>,
}
impl Action {
	pub fn execute(&self, result: &Workspace) {
		match &self.handler {
			Some(handler) => {
				handler(result);
			}
			None => {
				eprintln!("No handler for {}", self.id);
			}
		}
	}
}
pub enum ActionCategory {
	Setup,
	Analysis,
	Navigation,
	Documentation,
	Dependencies,
	Estate,
	Git,
	Build,
}

pub struct ActionRegistry;
impl ActionRegistry {
	pub fn from_analysis(_result: &Workspace) -> Vec<Action> {
		let mut actions = Vec::new();
		actions.extend(analysis_actions());
		actions.extend(bootstrap_actions());
		actions.extend(dependency_actions());
		actions.extend(documentation_actions());
		actions.extend(navigation_actions());
		actions.extend(estate_actions());
		actions
	}
}
pub type Handler = fn(&Workspace);
pub fn bootstrap_actions() -> Vec<Action> {
	vec![
		Action {
			id: "estate.init".into(),
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			title: "Initialize Estate".into(),
			description: "Create an .estate workspace and configuration.".into(),
			enabled: true,
		},
		Action {
			id: "workspace.scan".into(),
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			title: "Scan Workspace".into(),
			description: "Discover packages, files, and dependencies.".into(),
			enabled: true,
		},
	]
}
pub fn analysis_actions() -> Vec<Action> {
	vec![
		Action {
			id: "analyze.workspace".into(),
			reason: None,
			category: ActionCategory::Analysis,
			title: "Analyze Workspace".into(),
			description: "Build the complete symbol graph.".into(),
			enabled: true,
			handler: Some(|analysis| {
				MetricsRenderer::render(analysis);
			}),
		},
		Action {
			id: "analysis.file".into(),
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			title: "Analyze Current File".into(),
			description: "Analyze only the selected source file.".into(),
			enabled: true,
		},
		Action {
			id: "analysis.metrics".into(),
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			title: "Show Metrics".into(),
			description: "Summarize files, symbols, functions, and types.".into(),
			enabled: true,
		},
	]
}
pub fn dependency_actions() -> Vec<Action> {
	vec![
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "deps.graph".into(),
			title: "Dependency Graph".into(),
			description: "Visualize package and module dependencies.".into(),
			enabled: true,
		},
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "deps.unused".into(),
			title: "Find Unused Dependencies".into(),
			description: "Locate dependencies that are never referenced.".into(),
			enabled: true,
		},
	]
}
pub fn documentation_actions() -> Vec<Action> {
	vec![
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "docs.render".into(),
			title: "Render Documentation".into(),
			description: "Generate project documentation.".into(),
			enabled: true,
		},
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "docs.metrics".into(),
			title: "Render Metrics".into(),
			description: "Generate a metrics report.".into(),
			enabled: true,
		},
	]
}
pub fn navigation_actions() -> Vec<Action> {
	vec![
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "nav.symbol".into(),
			title: "Jump to Symbol".into(),
			description: "Locate a declaration by name.".into(),
			enabled: true,
		},
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "nav.references".into(),
			title: "Find References".into(),
			description: "Show every use of a symbol.".into(),
			enabled: true,
		},
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "nav.callers".into(),
			title: "Show Callers".into(),
			description: "Display functions that call the current symbol.".into(),
			enabled: true,
		},
	]
}
pub fn estate_actions() -> Vec<Action> {
	vec![
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "estate.sync".into(),
			title: "Sync Estate".into(),
			description: "Update Estate metadata from the workspace.".into(),
			enabled: true,
		},
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "estate.repair".into(),
			title: "Repair Estate".into(),
			description: "Fix missing or inconsistent metadata.".into(),
			enabled: true,
		},
		Action {
			handler: None,
			reason: None,
			category: ActionCategory::Analysis,
			id: "estate.validate".into(),
			title: "Validate Estate".into(),
			description: "Check the workspace for structural problems.".into(),
			enabled: true,
		},
	]
}
///--------------------------------------------------------------------------------
/// CLI UI Rendering
///--------------------------------------------------------------------------------
pub struct MetricsRenderer;
impl MetricsRenderer {
	// fn render_workspace(workspace: &Workspace) {
	//     let metrics = workspace.workspace_metrics();
	//     let mut table = ConsoleTheme::table(&["Metric", "Count"]);
	//     table.add_row(vec!["Files", &metrics.files.to_string()]);
	//     table.add_row(vec!["Packages", &metrics.packages.to_string()]);
	//     table.add_row(vec!["Symbols", &metrics.symbols.to_string()]);
	//     table.add_row(vec!["Functions", &metrics.functions.to_string()]);
	//     table.add_row(vec!["Types", &metrics.types.to_string()]);
	//     println!("\nWorkspace");
	//     println!("{table}");
	// }
	pub fn render(workspace: &Workspace) {
		Self::summary(
			"Workspace",
			vec![
				("Files", workspace.files.len()),
				("Packages", workspace.packages.len()),
				("Symbols", workspace.symbols.len()),
			],
		);
		Self::table(
			"Files",
			&["Name", "Symbols", "Functions", "Types", "Imports"],
			workspace
				.metrics()
				.files
				.iter()
				.map(|m| {
					vec![
						m.name.clone(),
						m.symbols.to_string(),
						m.functions.to_string(),
						m.types.to_string(),
						m.imports.to_string(),
					]
				})
				.collect(),
		);
		Self::table(
			"Packages",
			&["Name", "Files", "Modules", "Functions", "Types", "Imports"],
			workspace
				.metrics()
				.packages
				.iter()
				.map(|m| {
					vec![
						m.name.clone(),
						m.files.to_string(),
						m.modules.to_string(),
						m.functions.to_string(),
						m.types.to_string(),
						m.imports.to_string(),
					]
				})
				.collect(),
		);
	}
	pub fn render_workspace(workspace: &Workspace) {
		let metrics = workspace.workspace_metrics();
		let mut table = Table::new();
		table.set_header(vec!["Metric", "Count"]);
		table.add_row(vec![Cell::new("Files"), Cell::new(metrics.files)]);
		table.add_row(vec![Cell::new("Packages"), Cell::new(metrics.packages)]);
		table.add_row(vec![Cell::new("Symbols"), Cell::new(metrics.symbols)]);
		table.add_row(vec![Cell::new("Functions"), Cell::new(metrics.functions)]);
		table.add_row(vec![Cell::new("Types"), Cell::new(metrics.types)]);
		table.add_row(vec![Cell::new("Imports"), Cell::new(metrics.imports)]);
		println!("Workspace\n");
		println!("{table}");
	}
	pub fn render_files(workspace: &Workspace) {
		let mut table = Table::new();
		table.set_header(vec!["#", "File", "Symbols", "Functions", "Types"]);
		for (index, id) in workspace.files.iter().enumerate() {
			let metrics = workspace.file_metrics(*id);
			table.add_row(vec![
				Cell::new(index + 1),
				Cell::new(metrics.name),
				Cell::new(metrics.symbols),
				Cell::new(metrics.functions),
				Cell::new(metrics.types),
			]);
		}
		println!("\nFiles\n");
		println!("{table}");
	}
	pub fn render_packages(workspace: &Workspace) {
		let mut table = Table::new();
		table.set_header(vec!["Package", "Files", "Symbols", "Functions", "Types"]);
		for id in &workspace.packages {
			let metrics = workspace.package_metrics(*id);
			table.add_row(vec![
				Cell::new(metrics.name),
				Cell::new(metrics.files),
				Cell::new(metrics.symbols),
				Cell::new(metrics.functions),
				Cell::new(metrics.types),
			]);
		}
		println!("\nPackages\n");
		println!("{table}");
	}
}
impl MetricsRenderer {
	fn summary(title: &str, items: Vec<(&str, impl ToString)>) {
		println!();
		println!("{title}");
		println!("{}", "-".repeat(40));
		for (name, value) in items {
			println!("{:<20} {}", name, value.to_string());
		}
	}
	fn table(title: &str, headers: &[&str], rows: Vec<Vec<String>>) {
		println!();
		println!("{title}");
		for header in headers {
			print!("{:<18}", header);
		}
		println!();
		println!("{}", "-".repeat(headers.len() * 18));
		for row in rows {
			for cell in row {
				print!("{:<18}", cell);
			}
			println!();
		}
	}
}
pub struct ConsoleTheme;
impl ConsoleTheme {
	pub fn table(headers: &[&str]) -> Table {
		let mut table = Table::new();
		table.load_preset(UTF8_FULL).set_header(headers);
		table
	}
}
