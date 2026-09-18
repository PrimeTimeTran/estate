pub use anyhow::{self, Error, Result};
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use futures::FutureExt;
pub use serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use serde_json::Value;
pub use std::{
	collections::*,
	env,
	fmt::{self, Debug, Display},
	fs::{self},
	marker::PhantomData,
	path::*,
	sync::{
		Arc, Mutex, OnceLock, RwLock, RwLockReadGuard,
		atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering},
	},
	time::{Duration, Instant, SystemTime},
};
pub use tokio::task::JoinHandle;
pub use tokio_util::sync::CancellationToken;
pub use uuid::Uuid;

/// ## Warning
///
/// Enabling warnings by removing the suppression macro in lib.rs makes this blow up with warnings.
///
/// Don't touch this unless all 3 platforms build & run using the smoke testing script below
///
/// [../script/git-precommit-hook.sh]
///
pub use crate::{
	// The app prefix prevents collision warnings from the ./src/app directory use/import of ./lib/mod.rs
	app_entry::*,
	app_macros,
	app_prelude::*,
	data::*,
	e,
	impls::{self, self as i},
	r#macro::*,
	model::*,
	runtime::*,
	service::*,
	share::{share_prelude::*, *},
	structs::{self, self as s, *},
	tool::{time::*, *},
	// Verbosely use/export intrinsic traits because of name collisions with external crates.
	traits::{self, self as t, Context, EventReceiver, *},
	ui::{config::*, prelude::*, theme::*, ui_trait::*, *},
};

// This gate auto imports native when appropriate saving multiple use statements.
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use crate::{
	app::{app_native::*, context::*, *},
	native::{prelude::*, state::*, *},
	server::{self, events::*, fs::*},
	tool::logger::*,
};

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use crate::web::{app::*, bridge::*, *};
