use crate::prelude::*;

pub mod agent;
pub mod app;
pub mod backend;
pub mod cursor;
pub mod daemon;
pub mod discovery;
pub mod job;
pub mod monitor;
pub mod observer;
pub mod poc;
#[path = "[prelude].rs"]
pub mod prelude_native;
pub mod resolver;
pub mod router;
pub mod runtime;
pub mod screens;
pub mod state;
pub mod task;
pub mod ui;
pub mod util;
pub mod window;

#[path = "./[trait].rs"]
pub mod native_traits;

pub use agent::*;
pub use cursor::*;
pub use prelude_native::*;
pub use util::*;

pub use native_traits::*;


mod gestures;

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
	type Persistence = NativePersistence;
	type Network = NativeNetwork;
	type Client = ApiClient;
	fn persistence(&self) -> &Self::Persistence {
		todo!("");
	}
	fn network(&self) -> &Self::Network {
		todo!("")
	}
	fn api(&self) -> &Option<Self::Client> {
		&self.api
	}
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
