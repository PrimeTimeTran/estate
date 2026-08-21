pub use crate::prelude::*;
use cli::prelude::{CliCommand, Context as CliContext, FormatArgs};
use revelation::analyzer::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
///--------------------------------------------------------------------------------
/// Daemon
///--------------------------------------------------------------------------------
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
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
		let pid = std::fs::read_to_string(PID_PATH).unwrap_or_else(|_| "unknown".to_string());
		println!("📊 Estate Daemon Status");
		println!("──────────────────────");
		println!("✅ Status:          OK");
		println!("🆔 PID:             {}", pid);
		println!("🚀 Starts:          {}", state.starts);
		println!("🔎 Status checks:   {}", state.status_checks);
		println!("🕒 Started at:      {}", state.started_at);
		println!("⏱ Longest run:     {}s", state.longest_run);
		match tokio::net::UnixStream::connect(SOCKET_PATH).await {
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
	pub async fn run(&self, args: &FormatArgs) {
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
		args: &cli::context::AnalyzeArgs,
	) -> Result<Workspace, AnalysisError> {
		let target_path = PathBuf::from(&args.paths[0]);
		let request = Analyze {
			target: AnalysisTarget::File(target_path.clone()),
			subject: None,
		};
		let analyzer = RustAnalyzer;
		let options = analyzer::AnalyzerOptions::default();
		let workspace = analyzer.analyze(request, &options)?;
		let _metrics = workspace.metrics();
		Ok(workspace)
	}
}
pub enum AnalysisRequest {
	AnalyzeWorkspace,
}
pub struct AnalyzeLoop {
	rx: mpsc::Receiver<AnalysisRequest>,
	workspace: Workspace,
}
impl AnalyzeLoop {
	pub async fn run(mut self) {
		let actions = ActionRegistry::from_analysis(&self.workspace);
		while let Some(request) = self.rx.recv().await {
			match request {
				AnalysisRequest::AnalyzeWorkspace => {
					println!("Processing workspace request & building AST...");
					let _analyze_action = actions.iter().find(|a| a.title == "analyze.workspace");
				}
			}
		}
	}
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
						action.execute(&workspace, ActionOptions::default());
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
