use anyhow::Context;
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_foundation::NSString;

use crate::{
	model::resolver::{FS, SpecialFile, crate_root},
	prelude::*,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum ContextSource {
	Macos,
}

use std::process::Command;

// https://github.com/openclaw/AXorcist
// brew install openclaw/tap/axorc
// which axorc
// axorc --help
// axorc permissions

// swift build -c release --product axorc
// install -m 755 .build/release/axorc /usr/local/bin/axorc

// local
// swift package resolve
// swift package edit Commander --path ../Commander
// # Develop against the local checkout, then restore the released dependency:
// swift package unedit Commander

pub struct Axorc {
	path: PathBuf,
}

// $ while true; do
//   echo '{"command_id":"focused","command":"getFocusedElement","attributes":["AXRole","AXTitle","AXValue","AXURL"]}' |
//     axorc raw --stdin
//   sleep 1
// done

// echo '{
//   "command_id": "focused-element",
//   "command": "getFocusedElement",
//   "attributes": ["AXRole", "AXTitle", "AXValue", "AXURL"]
// }' | axorc raw --stdin | jq '.data | {role: .AXRole, title: .AXTitle, value: .AXValue, url: .AXURL}'

impl Axorc {
	pub fn new() -> Result<Self> {
		// active apps only
		// $ axorc tree --app "Google Chrome" --depth 2

		let output = Command::new("which")
			.arg("axorc")
			.output()
			.context("failed to locate `axorc`")?;

		if !output.status.success() {
			anyhow::bail!("`axorc` was not found on PATH");
		}

		let path = PathBuf::from(String::from_utf8(output.stdout)?.trim());

		tracing::info!(
			path = %path.display(),
			"found axorc"
		);

		Ok(Self { path })
	}
	pub fn init() -> Result<Axorc> {
		let axorc = Axorc::new()?;
		axorc.permissions()?;

		Ok(axorc)
	}
	pub fn permissions(&self) -> Result<()> {
		let output = Command::new(&self.path)
			.arg("permissions")
			.output()
			.context("failed to execute `axorc permissions`")?;

		let stdout = String::from_utf8_lossy(&output.stdout);
		let stderr = String::from_utf8_lossy(&output.stderr);

		tracing::info!(
			status = ?output.status,
			"axorc permissions completed"
		);

		if !stdout.is_empty() {
			tracing::info!(stdout = %stdout.trim(), "axorc");
		}

		if !stderr.is_empty() {
			tracing::warn!(stderr = %stderr.trim(), "axorc");
		}

		if !output.status.success() {
			anyhow::bail!("axorc permissions check failed");
		}

		Ok(())
	}
}

pub trait ContextProvider {
	fn snapshot(&self) -> Result<Snapshot>;
}
pub trait HostContextProvider: ContextProvider {}
pub trait BrowserContextProvider: ContextProvider {}
pub trait ExtensionContextProvider: ContextProvider {}

impl<C> App<C>
where
	C: Ctx,
{
	pub fn init_daemon(&mut self) -> Result<()> {
		tracing::info!("init_daemon");

		let origin = Self::detect_context_app()?;
		tracing::info!(?origin, "detected context application");

		let provider = MacosHostContextProvider::new(origin);
		let _axorc_app = Axorc::init()?;
		let cancel = self.cancel.clone();
		let runtime = tokio::runtime::Runtime::new()?;

		runtime.block_on(
			HostContextWatcher::new(provider)
				.interval(Duration::from_millis(250))
				.run(cancel),
		)?;

		Ok(())
	}

	fn context_origin(app: &NSRunningApplication) -> Option<ContextOrigin> {
		let bundle_id = app.bundleIdentifier()?.to_string();

		match bundle_id.as_str() {
			"com.microsoft.VSCode" => Some(ContextOrigin::VSCode),
			"dev.zed.Zed" => Some(ContextOrigin::Zed),
			"com.jetbrains.rustrover" => Some(ContextOrigin::RustRover),
			"net.kovidgoyal.kitty" => Some(ContextOrigin::Kitty),
			_ => None,
		}
	}

	fn detect_context_app() -> Result<ContextOrigin> {
		let mut pid = std::process::id();

		loop {
			let ppid = Self::parent_pid(pid)?;

			if ppid == 0 || ppid == pid {
				anyhow::bail!("could not find a known host application in process tree");
			}

			pid = ppid;

			let Some(app) =
				NSRunningApplication::runningApplicationWithProcessIdentifier(pid as libc::pid_t)
			else {
				continue;
			};

			let bundle_id = app.bundleIdentifier().map(|value| value.to_string());
			let name = app.localizedName().map(|value| value.to_string());

			tracing::debug!(pid, ?bundle_id, ?name, "found application in process tree");

			if let Some(origin) = Self::context_origin(&app) {
				return Ok(origin);
			}

			// This is an application, but not one we recognize as a
			// valid context origin. Keep walking the process tree.
		}
	}

	fn parent_pid(pid: u32) -> Result<u32> {
		let output = Command::new("ps")
			.args(["-p", &pid.to_string(), "-o", "ppid="])
			.output()
			.context("failed to inspect process parent")?;

		if !output.status.success() {
			anyhow::bail!(
				"failed to get parent PID for process {pid}: {}",
				String::from_utf8_lossy(&output.stderr).trim()
			);
		}
		let stdout = String::from_utf8(output.stdout).context("`ps` returned invalid UTF-8")?;
		stdout
			.trim()
			.parse::<u32>()
			.with_context(|| format!("invalid parent PID for process {pid}: {:?}", stdout.trim()))
	}
}

fn collect_host_context(provider: &impl HostContextProvider) -> Result<Snapshot> {
	provider.snapshot()
}

pub fn save_host_context(provider: &impl HostContextProvider) -> Result<()> {
	let snapshot = collect_host_context(provider)?;
	let path = SpecialFile::HostContext.path()?;
	tracing::info!(
		path = %path.display(),
		"writing host context"
	);
	FS::save(path, &snapshot)
}

fn find_application(workspace: &NSWorkspace, bundle_id: &str) -> Option<PathBuf> {
	let bundle_id = NSString::from_str(bundle_id);

	workspace
		.URLForApplicationWithBundleIdentifier(&bundle_id)
		.and_then(|url| url.path())
		.map(|path| PathBuf::from(path.to_string()))
}
pub fn host_context_path() -> PathBuf {
	crate_root().join(".estate/context/host.json")
}
fn installed_apps(workspace: &NSWorkspace) -> Vec<InstalledApp> {
	[
		("vscode", "com.microsoft.VSCode"),
		("zed", "dev.zed.Zed"),
		("rustrover", "com.jetbrains.rustrover"),
		("chrome", "com.google.Chrome"),
	]
	.into_iter()
	.filter_map(|(name, bundle_id)| {
		find_application(workspace, bundle_id).map(|path| InstalledApp::new(name, bundle_id, path))
	})
	.collect()
}
fn running_apps(workspace: &NSWorkspace) -> Vec<serde_json::Value> {
	workspace
		.runningApplications()
		.iter()
		.filter_map(|app| running_app(&app))
		.collect()
}
fn running_app(app: &NSRunningApplication) -> Option<serde_json::Value> {
	let pid = app.processIdentifier();
	if pid < 0 {
		return None;
	}
	let bundle_id = app.bundleIdentifier().map(|value| value.to_string());
	let name = app.localizedName().map(|value| value.to_string());
	let executable = app
		.executableURL()
		.and_then(|url| url.path())
		.map(|path| path.to_string());
	let launch_date = app.launchDate().map(|date| date.description().to_string());
	Some(serde_json::json!({
		"name": name,
		"bundle_id": bundle_id,
		"pid": pid,
		"executable": executable,
		"launch_date": launch_date,
		"active": app.isActive(),
		"hidden": app.isHidden(),
		"finished_launching": app.isFinishedLaunching(),
		"terminated": app.isTerminated(),
		"architecture": app.executableArchitecture(),
	}))
}

fn frontmost_app(workspace: &NSWorkspace) -> serde_json::Value {
	let Some(app) = workspace.frontmostApplication() else {
		return serde_json::Value::Null;
	};
	serde_json::json!({
		"name": app.localizedName().map(|value| value.to_string()),
		"bundle_id": app.bundleIdentifier().map(|value| value.to_string()),
		"pid": app.processIdentifier(),
		"launch_date": app
			.launchDate()
			.map(|value| value.description().to_string()),
		"executable": app
			.executableURL()
			.and_then(|url| url.path())
			.map(|path| path.to_string()),
	})
}
impl<P> HostContextWatcher<P>
where
	P: HostContextProvider,
{
	pub fn new(provider: P) -> Self {
		Self {
			provider,
			interval: Duration::from_millis(250),
		}
	}

	pub fn interval(mut self, interval: Duration) -> Self {
		self.interval = interval;
		self
	}

	pub async fn run(self, cancel: CancellationToken) -> Result<()> {
		let started_at = Utc::now();
		let mut writes = 0;

		loop {
			let snapshot = self.provider.snapshot()?;

			writes += 1;

			let now = Utc::now();

			let snapshot = ContextSnapshot {
				source: snapshot.source,
				timestamp: snapshot.timestamp,
				session: ContextSession {
					started_at,
					last_write: now,
					writes,
				},
				data: snapshot.data,
			};

			let path = SpecialFile::HostContext.path()?;

			tracing::info!(
					path = %path.display(),
					focused = ?snapshot.data["focused"],
					writes = snapshot.session.writes,
					"writing host context"
			);

			FS::save(path, &snapshot)?;

			tokio::select! {
					_ = cancel.cancelled() => return Ok(()),
					_ = tokio::time::sleep(Duration::from_secs(5)) => {}
			}
		}
	}
}
impl InstalledApp {
	pub fn new(name: &str, bundle_id: &str, path: PathBuf) -> Self {
		Self {
			name: name.to_owned(),
			bundle_id: bundle_id.to_owned(),
			path,
		}
	}
}

#[cfg(target_os = "macos")]
use axuielement::prelude::*;

impl MacosHostContextProvider {
	fn focused(&self) -> Result<serde_json::Value> {
		let Some(system) = system_wide() else {
			anyhow::bail!("failed to create system-wide AX element");
		};

		let Some(ax_app) = system.focused_application()? else {
			anyhow::bail!("AX system has no focused application");
		};

		let pid = ax_app.pid()?;

		let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) else {
			anyhow::bail!("no NSRunningApplication for focused PID {pid}");
		};

		Ok(serde_json::json!({
				"name": app.localizedName().map(|value| value.to_string()),
				"bundle_id": app.bundleIdentifier().map(|value| value.to_string()),
				"pid": pid,
				"executable": app
						.executableURL()
						.and_then(|url| url.path())
						.map(|path| path.to_string()),
		}))
	}
	fn installed(&self) -> Result<serde_json::Value> {
		let workspace = NSWorkspace::sharedWorkspace();

		Ok(serde_json::to_value(installed_apps(&workspace))?)
	}

	fn running(&self) -> Result<serde_json::Value> {
		let workspace = NSWorkspace::sharedWorkspace();

		Ok(serde_json::Value::Array(running_apps(&workspace)))
	}

	fn frontmost(&self) -> Result<serde_json::Value> {
		let workspace = NSWorkspace::sharedWorkspace();
		Ok(frontmost_app(&workspace))
	}

	fn system(&self) -> Result<serde_json::Value> {
		let process_info = objc2_foundation::NSProcessInfo::processInfo();

		Ok(serde_json::json!({
			"cpu_count": process_info.processorCount(),
			"physical_memory": process_info.physicalMemory(),
			"uptime": process_info.systemUptime(),
		}))
	}
}

impl MacosHostContextProvider {
	pub fn new(app: ContextOrigin) -> Self {
		Self { app }
	}
}
impl ContextProvider for MacosHostContextProvider {
	fn snapshot(&self) -> Result<Snapshot> {
		Ok(Snapshot {
			source: ContextSource::Macos,
			timestamp: Utc::now(),
			data: serde_json::json!({
					"host": self.system()?,
					"installed": self.installed()?,
					"app": self.app,
					"focused": self.focused()?,
			}),
		})
	}
}

impl HostContextProvider for MacosHostContextProvider {}

pub struct ApplicationContext;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContextOrigin {
	VSCode,
	Zed,
	RustRover,
	Kitty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextSession {
	pub started_at: DateTime<Utc>,
	pub last_write: DateTime<Utc>,
	pub writes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextSnapshot {
	pub source: ContextSource,
	pub timestamp: DateTime<Utc>,
	pub session: ContextSession,
	pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
	pub source: ContextSource,
	pub timestamp: DateTime<Utc>,
	pub data: serde_json::Value,
}

pub struct HostContext {
	pub timestamp: DateTime<Utc>,
	pub frontmost: Option<ApplicationContext>,
	pub running: Vec<ApplicationContext>,
	pub installed: InstalledApplications,
}
pub struct HostContextWatcher<P> {
	provider: P,
	interval: Duration,
}
pub struct InstalledApplications;

#[derive(serde::Serialize)]
pub struct InstalledApp {
	name: String,
	bundle_id: String,
	path: PathBuf,
}
pub struct MacosHostContextProvider {
	app: ContextOrigin,
}
pub struct MacosAccessibilityContext;

pub struct FocusedContext {
	// pub application: Application,
	pub role: Option<String>,
	pub title: Option<String>,
	pub value: Option<String>,
	pub url: Option<String>,
	pub attributes: HashMap<String, serde_json::Value>,
}

// estate context watch --key foo --launch-app "zed terminal"
// estate context watch --key bar --launch-app "vscode terminal"

// ## bash script
// reveals shared terminal internals.
// - reveals inconsistent other stuff... but good start.

// while true; do
//   echo '{"command_id":"focused","command":"getFocusedElement","attributes":["AXRole","AXTitle","AXValue","AXURL"]}' |
//     axorc raw --stdin |
//     jq -r '
//       .data |
//       "────────────────────────────────────────",
//       "ROLE:    \(.role // "-")",
//       "TITLE:   \(.attributes.AXTitle.any_value // "-")",
//       "VALUE:   \(.attributes.AXValue.any_value // "-")",
//       "URL:     \(.attributes.AXURL.any_value // "-")",
//       "CONTENT: \(.textual_content // "-")",
//       "PATH:    \((.path // []) | last // "-")"
//     '
//   sleep 5
// done

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextProbe {
	pub launch_app: String,
	pub cli_command: String,
	pub last_write_timestamp: DateTime<Utc>,
	pub last_write_focus: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextProbes {
	pub probes: HashMap<String, ContextProbe>,
}
