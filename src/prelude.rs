pub use anyhow::{self, Error, Result};
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use futures::FutureExt;
pub use serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use serde_json::Value;
pub use std::{
	collections::*,
	env,
	fmt::{self, Debug},
	fs::{self},
	marker::PhantomData,
	path::*,
	sync::{
		Arc, Mutex, OnceLock, RwLock,
		atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering},
	},
	time::{Duration, Instant, SystemTime},
};
pub use tokio::task::JoinHandle;
pub use tokio_util::sync::CancellationToken;
pub use uuid::Uuid;

/// ## Warning
///
/// Disabling lint suppressions in lib.rs makes this blow up.
///
/// Don't touch this unless all 3 platforms have been built & run using the following script.
///
/// [../script/git-precommit-hook.sh]
///
pub use crate::{
	api::*,
	app::app_entry::{self, *},
	app_macros,
	app_prelude::*,
	data::*,
	e, impls,
	r#macro::*,
	model::*,
	runtime::*,
	services::*,
	share::{share_prelude::*, *},
	structs::{self, *},
	tool::{time::*, *},
	traits,
	traits::{Context, EventReceiver, *},
	ui::{config::*, theme::*, ui_prelude::*, ui_trait::*, *},
};

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use crate::web::*;
// #[cfg(all(feature = "web", target_arch = "wasm32"))]
// pub use crate::{app::app_web::*, web::*};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use crate::{
	app::{app_native::*, context::*, *},
	logger::*,
	native::{native_prelude::*, *},
	native_state::*,
	server::{self, events::*},
};
