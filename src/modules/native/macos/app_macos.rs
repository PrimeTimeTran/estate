use super::*;
use crate::prelude::*;

impl<C> App<C>
where
	C: Ctx,
{
	pub fn init_daemon(&mut self) -> Result<()> {
		tracing::info!("🕋 init_daemon");
		Ok(())
	}
}

impl App<Context> {
	pub fn api(&self) -> &ApiService {
		self.host.api()
	}
	pub fn run(&mut self) -> Result<()> {
		self.init_services()?;
		self.run_gui()?;
		if self.mode == AppMode::Daemon {
			let hid = self.host.start()?;
			self.workers.push(hid);
		}
		Ok(())
	}
	pub fn run_gui(&mut self) -> Result<()> {
		let cancel = CancellationToken::new();
		let event_loop = EventLoop::<AppEvent>::with_user_event()
			.build()
			.expect("failed to build GUI event loop");
		let proxy = event_loop.create_proxy();
		let handle = self.start_app_events(proxy.clone())?;
		self.workers.push(handle);
		let event_rx = self.host.event_bus.subscribe_broadcast("app");
		let event_tx = self.host.event_bus.sender();
		self.host.runtime.attach_event_proxy(proxy);

		tracing::info!("app macos");
		// #[cfg(not(feature = "daemon"))]
		{
			let mut renderer = Renderer::<Context, <Context as Ctx>::AppState>::new(
				self.host.context(),
				self.state.clone(),
				Arc::new(self.settings.clone()),
				cancel,
				event_rx,
				event_tx,
			);
			event_loop
				.run_app(&mut renderer)
				.map_err(|err| anyhow::anyhow!("GUI event loop failed: {err}"))?;
		}
		Ok(())
	}
	// pub async fn run_daemon(&mut self) -> Result<()> {
	// 	let ipc = IpcServer::new(ESTATE_IPC_SOCKET, self.host.event_bus.clone());
	// 	let _handle = tokio::spawn(async move { ipc.start().await });
	// 	// self.workers.push(handle);
	// 	Ok(())
	// }
}

impl From<NativeEventKind> for e::EventKind {
	fn from(native: NativeEventKind) -> Self {
		match native {
			NativeEventKind::KeyDown { key_code } => e::EventKind::KeyDown { key_code },
			NativeEventKind::KeyUp { key_code } => e::EventKind::KeyUp { key_code },
			NativeEventKind::MouseDown { button, x, y } => e::EventKind::MouseDown { button, x, y },
			NativeEventKind::MouseUp { button, x, y } => e::EventKind::MouseUp { button, x, y },
			NativeEventKind::Scroll {
				vertical,
				horizontal,
			} => e::EventKind::Scroll {
				vertical,
				horizontal,
			},
			NativeEventKind::FlagsChanged { key_code } => e::EventKind::FlagsChanged { key_code },
			_ => {
				// Safe fallback while debugging inbound events.
				e::EventKind::Unknown
			}
		}
	}
}
impl TryFrom<NativeEvent> for e::Event {
	type Error = ();
	fn try_from(native: NativeEvent) -> Result<Self, Self::Error> {
		let kind = if let Some(app) = native.frontmost_app {
			e::EventKind::ActiveAppChanged {
				name: app.name,
				bundle_id: app.bundle_id,
				pid: app.pid as u32,
			}
		} else {
			match native.kind {
				NativeEventKind::KeyDown { key_code } => e::EventKind::KeyDown { key_code },
				NativeEventKind::KeyUp { key_code } => e::EventKind::KeyUp { key_code },
				NativeEventKind::MouseDown { button, x, y } => e::EventKind::MouseDown { button, x, y },
				NativeEventKind::MouseUp { button, x, y } => e::EventKind::MouseUp { button, x, y },
				NativeEventKind::Scroll {
					vertical,
					horizontal,
				} => e::EventKind::Scroll {
					vertical,
					horizontal,
				},

				NativeEventKind::FlagsChanged { key_code } => e::EventKind::FlagsChanged { key_code },

				_ => return Err(()),
			}
		};

		Ok(e::Event {
			id: 0,
			kind,
			source: EventSource::Daemon,
			timestamp: native.sent_at,
		})
	}
}

pub struct AppEventBridge {
	rx: BroadcastReceiver<AppEvent>,
	tx: EventSender<AppEvent>,
	proxy: EventLoopProxy<AppEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeEvent {
	pub sent_at: u64,

	#[serde(flatten)]
	pub kind: NativeEventKind,

	#[serde(default)]
	pub modifiers: ModifierSnapshot,

	#[serde(default)]
	pub source: Option<String>,

	#[serde(default)]
	pub timestamp: Option<u64>,

	#[serde(default)]
	pub key_code: Option<u16>,

	#[serde(default)]
	pub name: Option<String>,

	pub scroll_x: Option<f64>,
	pub scroll_y: Option<f64>,

	#[serde(default)]
	pub direction: Option<keymap::KeyDirection>,

	#[serde(default)]
	pub frontmost_app: Option<FrontmostApp>,
}
