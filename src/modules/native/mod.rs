use crate::prelude::*;

pub mod agent;
pub mod backend;
pub mod cursor;
pub mod daemon;
pub mod discovery;

#[path = "./[impl].rs"]
pub mod native_impl;
pub use native_impl::*;

#[path = "./[trait].rs"]
pub mod native_traits;
pub use native_traits::*;

pub mod monitor;
pub mod poc;
#[path = "[prelude].rs"]
pub mod prelude_native;
pub mod renderer;
pub mod resolver;
pub mod router;
pub mod runtime;
pub mod screens;
pub mod state;
pub mod task;
pub mod ui;
pub mod util;

pub mod r#const;
pub use crate::modules::r#const::*;

pub use agent::*;
pub use cursor::*;
pub use prelude_native::*;
pub use renderer::*;
pub use util::*;

// This compiles the module only if the "windows" feature is enabled AND the OS is Windows
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(unix)]
pub mod unix;

#[cfg(unix)]
pub use unix::*;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
	FrontmostApp {
		name: String,
		#[serde(rename = "bundleID")]
		bundle_id: String,
		pid: i64,
	},
}

impl NativeHost {
	fn new() -> Self {
		Self::default()
	}
	fn run() -> Self {
		todo!("")
		// Self::default()
	}
}

impl NativeServices {
	pub async fn connect() -> anyhow::Result<Self> {
		let api = ApiClient::connect().await?;
		Ok(Self {
			persistence: NativePersistence::default(),
			network: NativeNetwork::default(),
			api: Some(api),
		})
	}
}

impl Network for NativeNetwork {
	fn is_available(&self) -> bool {
		todo!("")
	}
}

impl Persistence for NativePersistence {
	fn load(&self, _key: &str) -> Result<Option<Vec<u8>>> {
		todo!("")
	}
	fn save(&self, _key: &str, _value: &[u8]) -> Result<()> {
		todo!("")
	}
}

impl Services for NativeServices {
	fn persistence(&self) -> &Self::Persistence {
		todo!("");
	}
	fn network(&self) -> &Self::Network {
		todo!("")
	}
	fn api(&self) -> &Option<Self::Client> {
		&self.api
	}
	type Client = ApiClient;
	type Network = NativeNetwork;
	type Persistence = NativePersistence;
}

#[derive(Debug, Default)]
pub struct NativeHost {
	window: NativeWindow,
	storage: NativeStorage,
}

#[derive(Debug, Default, Clone)]
pub struct NativeNetwork;

#[derive(Debug, Default, Clone)]
pub struct NativePersistence;

#[derive(Debug, Default, Clone)]
pub struct NativeStorage;

#[derive(Clone, Debug, Default)]
pub struct NativeServices {
	persistence: NativePersistence,
	network: NativeNetwork,
	api: Option<ApiClient>,
}

#[derive(Debug, Default, Clone)]
pub struct NativeWindow;
