pub use crate::prelude::*;
pub use comfy_table::{Cell, Table, presets::UTF8_FULL};

///--------------------------------------------------------------------------------
/// CLI Tools
///--------------------------------------------------------------------------------
#[derive(Debug, Default)]
pub struct ActionOptions(HashMap<String, Value>);
impl ActionOptions {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn get(&self, key: &str) -> Option<&Value> {
		self.0.get(key)
	}
	pub fn insert(&mut self, key: impl Into<String>, value: Value) {
		self.0.insert(key.into(), value);
	}
}
pub struct Action {
	pub id: String,
	pub title: String,
	pub description: String,
	pub enabled: bool,
	pub reason: Option<String>,
	pub category: ActionCategory,
	pub handler: Option<Handler>,
}
pub type Handler = fn(&Workspace, &ActionOptions);
impl Action {
	pub fn execute(&self, workspace: &Workspace, options: ActionOptions) {
		match self.handler {
			Some(handler) => handler(workspace, &options),
			None => eprintln!("No handler for {}", self.id),
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
pub struct ActionRegistry {
	actions: Vec<Action>,
}
impl Default for ActionRegistry {
	fn default() -> Self {
		let actions = Vec::new();
		Self { actions }
	}
}
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
fn bootstrap_actions() -> Vec<Action> {
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
fn analysis_actions() -> Vec<Action> {
	vec![
		Action {
			id: "analyze.workspace".into(),
			reason: None,
			category: ActionCategory::Analysis,
			title: "Analyze Workspace".into(),
			description: "Build the complete symbol graph.".into(),
			enabled: true,
			handler: Some(|analysis, _options| {
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
fn dependency_actions() -> Vec<Action> {
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
fn documentation_actions() -> Vec<Action> {
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
fn navigation_actions() -> Vec<Action> {
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
fn estate_actions() -> Vec<Action> {
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
pub struct DaemonClient {
	socket_path: &'static str,
}
impl DaemonClient {
	pub fn new() -> Self {
		Self {
			socket_path: SOCKET_PATH,
		}
	}
	pub async fn execute(&self, action: ActionRequest) -> anyhow::Result<DaemonResponse> {
		let mut stream = UnixStream::connect(self.socket_path).await?;
		let request = serde_json::to_string(&action)?;
		stream.write_all(request.as_bytes()).await?;
		stream.write_all(b"\n").await?;
		let mut buf = Vec::new();
		stream.read_to_end(&mut buf).await?;
		let response: DaemonResponse = serde_json::from_slice(&buf)?;
		Ok(response)
	}
}
