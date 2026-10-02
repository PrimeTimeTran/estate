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

pub mod keymap;
pub use keymap::*;

pub mod hid;
pub use hid::*;

pub mod scroll;
pub use scroll::*;

pub mod window;
pub use window::*;

pub mod server;
pub use server::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum NativeEventKind {
	#[serde(rename = "key_down")]
	KeyDown {
		key_code: u16,
	},

	#[serde(rename = "key_up")]
	KeyUp {
		key_code: u16,
	},

	#[serde(rename = "mouse_down")]
	MouseDown {
		button: i64,
		x: f64,
		y: f64,
	},

	#[serde(rename = "mouse_up")]
	MouseUp {
		button: i64,
		x: f64,
		y: f64,
	},

	#[serde(rename = "scroll")]
	Scroll {
		vertical: i64,
		horizontal: i64,
	},

	#[serde(rename = "flags_changed")]
	FlagsChanged {
		key_code: u16,
	},

	ModifierChanged,
	#[serde(rename = "frontmost_app")]
	FrontmostApp,
}

impl App<Context> {
	pub fn api(&self) -> &ApiService {
		self.host.api()
	}
	pub fn run(&mut self) -> Result<()> {
		tracing::debug!("🍏 App run");
		self.init_services()?;
		self.run_gui()?;
		if self.mode == AppMode::Daemon {
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
impl Context {
	fn new(state: NativeState, api: ApiService) -> Self {
		Self { state, api }
	}
}
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

			NativeEventKind::FlagsChanged { key_code } => e::EventKind::FlagsChanged { key_code },

			_ => {
				// Safe fallback while debugging inbound events.
				e::EventKind::Unknown
			}
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
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

	#[serde(default)]
	pub direction: Option<keymap::KeyDirection>,
}

#[derive(Clone)]
pub struct Context {
	pub state: NativeState,
	pub api: ApiService,
}
