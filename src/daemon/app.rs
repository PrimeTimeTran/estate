use crate::prelude::*;
use revelation::analyzer::Workspace;
use tokio::runtime::Runtime;

use tray_icon::{
	Icon, TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuEvent, MenuItem},
};

use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::{ActiveEventLoop, EventLoop},
	platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS},
};

pub struct App {
	tray: TrayIcon,
	status: MenuItem,
	quit: MenuItem,
	context: Context,
	engine: EstateEngine,
	daemon_tx: Option<mpsc::Sender<DaemonCommand>>,
}
impl App {
	fn new(context: Context, engine: EstateEngine) -> anyhow::Result<Self> {
		let (status, quit, tray) = Self::bootstrap()?;

		Ok(Self {
			context,
			engine,
			tray,
			status,
			quit,
			daemon_tx: None,
		})
	}
	fn bootstrap() -> anyhow::Result<(MenuItem, MenuItem, TrayIcon)> {
		let menu = Menu::new();
		let status = MenuItem::new("● Estate Daemon Running", false, None);
		let quit = MenuItem::new("Quit", true, None);
		menu.append(&status)?;
		menu.append(&quit)?;
		let icon = Self::tray_icon();
		let tray = TrayIconBuilder::new()
			.with_icon(icon)
			.with_menu(Box::new(menu))
			.with_tooltip("Estate Daemon — Running")
			.build()
			.map_err(|e| anyhow::anyhow!("failed to create tray icon: {e}"))?;
		Ok((status, quit, tray))
	}
	fn tray_icon() -> Icon {
		let image = image::load_from_memory(constants::TRAY_ICON)
			.expect("failed to load generated tray icon")
			.into_rgba8();
		let (width, height) = image.dimensions();
		Icon::from_rgba(image.into_raw(), width, height).expect("failed to create tray icon")
	}
}
impl App {
	pub async fn spawn_tray_process() -> anyhow::Result<()> {
		if BackgroundDaemon::is_running().await {
			return Ok(());
		}

		let exe = std::env::current_exe()?;

		std::process::Command::new(exe)
			.arg("tray")
			.stdin(std::process::Stdio::null())
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.spawn()?;

		Ok(())
	}

	/// Run the tray application in the current process.
	pub fn run_tray_daemon(engine: EstateEngine) -> anyhow::Result<()> {
		let context = Context::new(RuntimeMode::Cli)?;

		let event_loop = EventLoop::builder()
			.with_activation_policy(ActivationPolicy::Accessory)
			.build()?;

		let mut app = Self::new(context, engine)?;

		event_loop.run_app(&mut app)?;

		Ok(())
	}
	fn run_daemon(engine: EstateEngine, mut rx: mpsc::Receiver<DaemonCommand>) {
		let runtime = Runtime::new().unwrap();

		runtime.block_on(async move {
			let mut daemon = BackgroundDaemon::new(engine);

			let daemon_task = tokio::spawn(async move {
				if let Err(e) = daemon.run_foreground().await {
					eprintln!("Daemon error: {e}");
				}
			});

			tokio::select! {
					result = daemon_task => {
							match result {
									Ok(()) => eprintln!("Daemon exited"),
									Err(e) => eprintln!("Daemon task failed: {e}"),
							}
					}

					command = rx.recv() => {
							match command {
									Some(DaemonCommand::Stop) => {
											eprintln!("Stopping daemon...");
									}
									None => {
											eprintln!("Daemon command channel closed");
									}
							}
					}
			}
		});
	}
}
impl ApplicationHandler for App {
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		if let Ok(event) = MenuEvent::receiver().try_recv() {
			if event.id() == self.quit.id() {
				if let Some(tx) = &self.daemon_tx {
					let _ = tx.send(DaemonCommand::Stop);
				}
				event_loop.exit();
			}
		}
	}
	fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
		let (tx, rx) = mpsc::channel(100);
		self.daemon_tx = Some(tx);
		let engine = self.engine.clone();
		thread::spawn(move || {
			Self::run_daemon(engine, rx);
		});
	}
	fn window_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_window_id: winit::window::WindowId,
		_event: WindowEvent,
	) {
	}
}

#[derive(Clone, Debug)]
pub struct Context {
	pub source: RuntimeMode,
	// Where the user is operating
	pub workspace: PathBuf,
	// Global user estate (~/.estate)
	pub estate_root: PathBuf,
	// Engine internals (cache, daemon state, registry)
	pub engine_root: PathBuf,
}
#[derive(Clone, Debug)]
pub enum RuntimeMode {
	Cli,
	Daemon,
	Lsp,
	Tray,
	// ZedEditor,
	// CompilerPipeline,
	// KnowledgeBase,
}
impl Context {
	pub fn new(source: RuntimeMode) -> std::io::Result<Self> {
		Ok(Self {
			source,
			workspace: std::env::current_dir()?,
			estate_root: crate::daemon::resolver::global_estate_dir()?,
			engine_root: crate::daemon::resolver::engine_data_dir()?,
		})
	}
}

// Command = "What does this executable invocation mean?"
// DaemonCommand = "What should happen to the running daemon?"
// ActionRequest = "What work should the daemon perform?"
//
// Things that can happen to the daemon itself
#[derive(Clone, Debug)]
enum DaemonCommand {
	Stop,
	// Metrics,
	// Restart,
	// Refresh,
	// Enable,
	// Disable,
	// Status,
}

// pub struct WorkspaceContext {
// 	pub root: PathBuf,
// 	pub estate: Option<PathBuf>,
// }

// pub struct RuntimeContext {
// 	pub engine_dir: PathBuf,
// 	pub connected: bool,
// }
