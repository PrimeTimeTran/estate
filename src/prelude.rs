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
pub use tokio::sync::broadcast::{self, error::TryRecvError};
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
	app_prelude::*,
	data::*,
	e,
	impls::{self, self as i},
	macros::{self, *},
	model::common::{Difficulty, Language},
	model::*,
	runtime::*,
	service::*,
	share::{share_prelude::*, *},
	structs::{self, self as s, *},
	// Verbosely use/export intrinsic traits because of name collisions with external crates.
	traits::{self, self as t, Context, EventReceiver, *},
	ui::{config::*, prelude_ui::*, theme::*, *},
	util::{time::*, *},
};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use crate::{
	app::{app_native::*, context::*, *},
	native::{
		job,
		prelude_native::*,
		runtime::*,
		state::*,
		util::{logger::*, *},
		*,
	},
	server::{self, channel, events::*, fs::*},
};

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use crate::web::{app::*, bridge::*, *};
