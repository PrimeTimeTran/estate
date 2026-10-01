use crate::prelude::*;

use anyhow::{Context as CtxAnyhow, Result};
pub use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes, kCFRunLoopDefaultMode};
pub use core_graphics::{
	display::{CGDisplay, CGPoint, CGRect},
	event::*,
	event_source::{CGEventSource, CGEventSourceStateID},
	geometry,
};

use mach2::mach_time;
pub use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

pub use host::*;
mod host;

pub mod scroll;
pub use scroll::*;

pub mod window;
pub use window::*;

pub mod server;
pub use server::*;

impl Ctx for Context {
	fn api(&self) -> &Self::Api {
		&self.api
	}
	fn api_mut(&mut self) -> &mut Self::Api {
		&mut self.api
	}
	fn initial_state() -> Self::AppState {
		structs::S {
			context: PhantomData,
			state: PhantomData,
			view: ViewType::MarkdownScreen,
		}
	}
	type Api = ApiService;
	type AppState = structs::S<Context>;
	type EventReceiver = structs::BroadcastReceiver<e::Event>;
	type EventSender = structs::BroadcastSender<e::Event>;
	type GuiState = NativeGuiState;
}

impl Context {
	fn new(state: NativeState, api: ApiService) -> Self {
		Self { state, api }
	}
}

impl Default for Context {
	fn default() -> Self {
		Self::new(NativeState::default(), ApiService::default())
	}
}

impl Host<Context> {
	pub fn init() -> anyhow::Result<Self> {
		let parsed = cli::context::parse();

		let mut config = LogConfig::load()?;
		config.apply_cli(&parsed);
		logger::init_logging(&config)?;

		// Create the one runtime.
		let tokio = tokio::runtime::Runtime::new()?;

		// Context is still uniquely owned here.
		let mut context = Context::default();

		// Daemon may not need this
		// This requires server access
		#[cfg(not(feature = "daemon"))]
		{
			// Connect using the same runtime that Host will retain.
			tokio.block_on(context.api_mut().connect())?;
		}

		// Only share Context after initialization.
		let context = Arc::new(context);
		Self::new(context, tokio)
	}
	fn logging() {}
}

impl App<Context> {
	pub fn api(&self) -> &ApiService {
		self.host.api()
	}
	pub fn run(&mut self) -> Result<()> {
		tracing::debug!("🍏 App run");
		tracing::info!("🍏 App run");
		self.init_services()?;
		self.run_gui()?;
		if self.mode == AppMode::Daemon {
			// self.init_daemon();
			let hid = self.host.start_hid()?;
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

		// #[cfg(not(feature = "daemon"))]
		{
			let mut renderer = structs::Renderer::<Context, <Context as Ctx>::AppState>::new(
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum NativeEventKind {
	#[serde(rename = "key_down")]
	KeyDown { key_code: u16 },

	#[serde(rename = "key_up")]
	KeyUp { key_code: u16 },

	#[serde(rename = "mouse_down")]
	MouseDown { button: i64, x: f64, y: f64 },

	#[serde(rename = "mouse_up")]
	MouseUp { button: i64, x: f64, y: f64 },

	#[serde(rename = "scroll")]
	Scroll { vertical: i64, horizontal: i64 },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NativeEvent {
	pub sent_at: u64,

	#[serde(flatten)]
	pub kind: NativeEventKind,
}

impl From<NativeEvent> for e::Event {
	fn from(native: NativeEvent) -> Self {
		Self {
			id: 0,
			kind: native.kind.into(),
			source: EventSource::Daemon,
			timestamp: native.sent_at,
		}
	}
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
		}
	}
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum HidMessage {
	#[serde(rename = "ping")]
	Ping { id: u64 },

	#[serde(rename = "pong")]
	Pong { id: u64 },

	#[serde(rename = "native_event")]
	NativeEvent { event: NativeEvent },

	#[serde(rename = "action")]
	Action { action: String },
}

pub struct MacosHid {
	socket: PathBuf,
}
fn mach_now() -> u64 {
	unsafe { mach_time::mach_absolute_time() }
}
impl MacosHid {
	pub fn new() -> Result<Self> {
		tracing::info!("MacosHidMacosHid new");
		Ok(Self {
			socket: PathBuf::from("/tmp/estate-hid.sock"),
		})
	}

	pub async fn run(self, events: EventBus, cancel: CancellationToken) -> Result<()> {
		tracing::info!(
				socket = %self.socket.display(),
				"connecting to Swift HID"
		);

		let stream = UnixStream::connect(&self.socket).await.with_context(|| {
			format!(
				"failed to connect to Swift HID socket {}",
				self.socket.display()
			)
		})?;

		tracing::info!("connected to Swift HID");

		let (reader, mut writer) = stream.into_split();
		let mut reader = BufReader::new(reader);

		let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(10));

		let mut ping_id = 0u64;
		let mut line = String::new();

		loop {
			tokio::select! {
					_ = cancel.cancelled() => {
							tracing::info!(
									"macOS HID cancelled"
							);

							return Ok(());
					}

					_ = ping_interval.tick() => {
							ping_id += 1;

							let ping = HidMessage::Ping {
									id: ping_id,
							};

							let json =
									serde_json::to_string(&ping)?;

							writer
									.write_all(json.as_bytes())
									.await?;

							writer
									.write_all(b"\n")
									.await?;

							tracing::info!(
									id = ping_id,
									"🍏 Rust → Swift: PING"
							);
					}
			result = reader.read_line(&mut line) => {
				tracing::info!("🔥 RUST READ WAKEUP");

				let received_at = mach_now();

				let bytes = result?;

				tracing::info!(
						bytes,
						"🔥 RUST READ COMPLETE"
				);

				if bytes == 0 {
						tracing::warn!("Swift HID disconnected");
						return Ok(());
				}

				let raw = line.trim_end();

				tracing::info!(
						raw,
						"🍎 Swift → Rust"
				);

							let message =
									match serde_json::from_str::<HidMessage>(
											raw
									) {
											Ok(message) => message,

											Err(error) => {
													tracing::error!(
															%error,
															raw,
															"invalid HID message from Swift"
													);

													line.clear();
													continue;
											}
									};

							match message {
									HidMessage::Ping { id } => {
											tracing::info!(
													id,
													"🍎 Swift → Rust: PING"
											);

											let pong =
													HidMessage::Pong { id };

											let json =
													serde_json::to_string(&pong)?;

											writer
													.write_all(json.as_bytes())
													.await?;

											writer
													.write_all(b"\n")
													.await?;

											tracing::info!(
													id,
													"🍏 Rust → Swift: PONG"
											);
									}

									HidMessage::Pong { id } => {
											tracing::info!(
													id,
													"🍎 Swift → Rust: PONG"
											);
									}

									HidMessage::NativeEvent { event } => {
											let latency =
													received_at
															.saturating_sub(event.sent_at);

											tracing::info!(
													latency,
													"🔥 RUST EVENT"
											);

											events.emit(event.into());
									}

									HidMessage::Action { action } => {
											tracing::info!(
													%action,
													"🍎 Swift action received"
											);
									}
							}

							line.clear();
					}
			}
		}
	}

	// pub async fn run(self, events: EventBus, cancel: CancellationToken) -> Result<()> {
	// tracing::info!(
	// socket = %self.socket.display(),
	// "connecting to Swift HID"
	// );
	//
	// let stream = UnixStream::connect(&self.socket).await.with_context(|| {
	// format!(
	// "failed to connect to Swift HID socket {}",
	// self.socket.display()
	// )
	// })?;
	//
	// tracing::info!("connected to Swift HID");
	//
	// let (reader, mut writer) = stream.into_split();
	// let mut reader = BufReader::new(reader);
	//
	// let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(10));
	//
	// let mut ping_id = 0u64;
	// let mut line = String::new();
	//
	// loop {
	// tokio::select! {
	// _ = cancel.cancelled() => {
	// tracing::info!("macOS HID cancelled");
	// return Ok(());
	// }
	//
	// _ = ping_interval.tick() => {
	// ping_id += 1;
	//
	// let ping = HidMessage::Ping {
	// id: ping_id,
	// };
	//
	// let json = serde_json::to_string(&ping)?;
	//
	// writer.write_all(json.as_bytes()).await?;
	// writer.write_all(b"\n").await?;
	// writer.flush().await?;
	//
	// tracing::info!(
	// id = ping_id,
	// "🍏 Rust → Swift: PING"
	// );
	// }
	//
	// // result = reader.read_line(&mut line) => {
	// // let bytes = result?;
	// //
	// // if bytes == 0 {
	// // tracing::warn!("Swift HID disconnected");
	// // return Ok(());
	// // }
	// //
	// // let message: HidMessage =
	// // serde_json::from_str(line.trim())
	// // .context("invalid HID message from Swift")?;
	// //
	// // match message {
	// // HidMessage::Ping { id } => {
	// // tracing::info!(
	// // id,
	// // "Swift → Rust: PING"
	// // );
	// //
	// // let pong = HidMessage::Pong { id };
	// // let json = serde_json::to_string(&pong)?;
	// //
	// // writer.write_all(json.as_bytes()).await?;
	// // writer.write_all(b"\n").await?;
	// // writer.flush().await?;
	// //
	// // tracing::info!(
	// // id,
	// // "🍏 Rust → Swift: PONG"
	// // );
	// // }
	// //
	// // HidMessage::Pong { id } => {
	// // tracing::info!(
	// // id,
	// // "Swift → Rust: PONG"
	// // );
	// // }
	// //
	// // HidMessage::NativeEvent { event } => {
	// // events.emit(event.into());
	// // }
	// //
	// // HidMessage::Action { action } => {
	// // tracing::info!(
	// // %action,
	// // "Swift requested Estate action"
	// // );
	// // }
	// // }
	// //
	// // line.clear();
	// // }
	// result = reader.read_line(&mut line) => {
	// let bytes = result?;
	//
	// if bytes == 0 {
	// tracing::warn!("Swift HID disconnected");
	// return Ok(());
	// }
	//
	// eprintln!(
	// "🔥 RUST RECEIVED {} bytes: {}",
	// bytes,
	// line.trim_end()
	// );
	//
	// let message: HidMessage =
	// serde_json::from_str(line.trim())
	// .context("invalid HID message from Swift")?;
	//
	// match message {
	// HidMessage::NativeEvent { event } => {
	// eprintln!("🔥 RUST NATIVE EVENT: {:?}", event);
	// events.emit(event.into());
	// }
	//
	// HidMessage::Ping { id } => {
	// eprintln!("PING {}", id);
	// }
	//
	// HidMessage::Pong { id } => {
	// eprintln!("PONG {}", id);
	// }
	//
	// HidMessage::Action { action } => {
	// eprintln!("ACTION {}", action);
	// }
	// }
	//
	// line.clear();
	// }
	// }
	// }
	// }
}

#[derive(Clone)]
pub struct Context {
	pub state: NativeState,
	pub api: ApiService,
}
