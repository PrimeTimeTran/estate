use revelation::analyzer::Workspace;
use std::path::PathBuf;
use std::thread;
use tokio::{
	runtime::Runtime,
	sync::mpsc::{Receiver, Sender, channel},
};
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

use crate::daemon::{app, start::BackgroundDaemon};

struct App {
	tray: Option<TrayIcon>,
	status: Option<MenuItem>,
	quit: Option<MenuItem>,
	context: app::Context,
	daemon_tx: Option<Sender<DaemonCommand>>,
}

#[derive(Clone, Debug)]
pub struct Context {
	pub source: ContextSource,

	// Where the user is operating
	pub workspace: PathBuf,

	// Global user estate (~/.estate)
	pub estate_root: PathBuf,

	// Engine internals (cache, daemon state, registry)
	pub engine_root: PathBuf,
}

#[derive(Clone, Debug)]
pub enum ContextSource {
	Cli,
	// ZedEditor,
	// CompilerPipeline,
	// KnowledgeBase,
}

impl Context {
	pub fn new(source: ContextSource) -> std::io::Result<Self> {
		Ok(Self {
			source,
			workspace: std::env::current_dir()?,
			estate_root: crate::daemon::resolver::global_estate_dir()?,
			engine_root: crate::daemon::resolver::engine_data_dir()?,
		})
	}
}

pub async fn run_tray_daemon() -> anyhow::Result<()> {
	let context = app::Context::new(app::ContextSource::Cli)?;
	let event_loop = EventLoop::builder()
		.with_activation_policy(ActivationPolicy::Accessory)
		.build()
		.unwrap();
	let mut app = App {
		context,
		tray: None,
		status: None,
		quit: None,
		daemon_tx: None,
	};
	event_loop.run_app(&mut app).unwrap();
	Ok(())
}

impl ApplicationHandler for App {
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		if let Ok(event) = MenuEvent::receiver().try_recv() {
			if Some(event.id()) == self.quit.as_ref().map(|item| item.id()) {
				if let Some(tx) = &self.daemon_tx {
					let _ = tx.send(DaemonCommand::Stop);
				}
				event_loop.exit();
			}
		}
	}
	fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
		let menu = Menu::new();
		let status = MenuItem::new("● Estate Daemon Running", false, None);
		let quit = MenuItem::new("Quit", true, None);
		menu.append(&status).unwrap();
		menu.append(&quit).unwrap();
		let icon = Icon::from_rgba(vec![255; 32 * 32 * 4], 32, 32).unwrap();
		let tray = TrayIconBuilder::new()
			.with_icon(icon)
			.with_menu(Box::new(menu))
			.with_tooltip("Estate Daemon — Running")
			.build()
			.unwrap();
		self.status = Some(status);
		self.quit = Some(quit);
		self.tray = Some(tray);
		let (tx, rx) = channel(100);
		self.daemon_tx = Some(tx);
		let context = self.context.clone();
		thread::spawn(move || {
			run_daemon(context, rx);
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

fn run_daemon(ctx: app::Context, mut rx: Receiver<DaemonCommand>) {
	let runtime = Runtime::new().unwrap();
	runtime.block_on(async move {
		let workspace = Workspace::new();
		let daemon = BackgroundDaemon::new(workspace, ctx);

		let mut daemon_task = tokio::spawn(async move {
			let mut daemon = daemon;

			if let Err(e) = daemon.run_foreground().await {
				eprintln!("Daemon error: {}", e);
			}
		});
		tokio::select! {
			result = &mut daemon_task => {
				match result {
					Ok(()) => eprintln!("Daemon exited"),
					Err(e) => eprintln!("Daemon task failed: {}", e),
				}
			}
			command = rx.recv() => {
				match command {
					Some(DaemonCommand::Stop) => {
						eprintln!("Stopping daemon...");
						daemon_task.abort();
						let _ = daemon_task.await;
						eprintln!("Daemon stopped");
					}
					None => {
						eprintln!("Daemon command channel closed");
						daemon_task.abort();
						let _ = daemon_task.await;
					}
					// _ => {
					// 	todo!("commands")
					// }
				}
			}
		}
	});
}
// Command = "What does this executable invocation mean?"
// DaemonCommand = "What should happen to the running daemon?"
// ActionRequest = "What work should the daemon perform?"
//
// Things that can happen to the daemon itself
#[derive(Debug)]
enum DaemonCommand {
	Stop,
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
