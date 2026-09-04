pub use ::serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use anyhow::{self, Error, Result};
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use futures::FutureExt;
pub use serde_json::Value;
pub use std::{
	collections::*,
	env,
	fmt::{self, Debug},
	fs::{self},
	path::*,
	sync::{
		Arc, Mutex, OnceLock, RwLock,
		atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering},
	},
	time::{Instant, SystemTime},
};
pub use uuid::Uuid;

pub use crate::{
	api::*,
	app::*,
	app_runtime::*,
	data::*,
	e,
	model::*,
	proto::{types, *},
	services::*,
	share::{prelude::*, *},
	theme::*,
	tool::{time::*, *},
	r#trait::*,
	ui::{config::*, r#trait::*, ui_prelude::*, *},
};

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use crate::app::app_web::*;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use crate::web::*;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use cli::context::*;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use crate::{
	app::{app_native::*, *},
	native::{prelude::*, *},
	server::{self, events::*},
};
