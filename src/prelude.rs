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
	pin::Pin,
	sync::{
		Arc, Mutex, OnceLock, RwLock, RwLockReadGuard,
		atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering},
	},
	time::{Duration, Instant, SystemTime},
};
pub use tokio::{
	sync::broadcast::{self, error::TryRecvError},
	task::JoinHandle,
};
pub use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};
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
	data::{configs::*, default::*},
	e,
	helper::*,
	impls::{self, self as i},
	macros::{self, *},
	// Verbose use/export to resolve name collisions with GRPC models.
	model::common::{Difficulty, Language},
	model::*,
	service::*,
	share::{share_prelude::*, *},
	structs::{self, self as s, *},
	// Verbose use/export to resolve name collisions with external crates.
	traits::{self, self as t, Context, EventReceiver, *},
	ui::{config::*, theme::*, *},
	util::{time::*, *},
};

#[cfg(not(target_arch = "wasm32"))]
pub use crate::{
	app::{context::*, *},
	modules::native::{
		job,
		runtime::*,
		state::*,
		util::{logger::*, *},
		*,
	},
	server::{self, channel, event::*, fs_deps::*},
};

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use crate::modules::web::{app::*, bridge::*, *};
