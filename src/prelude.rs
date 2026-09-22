pub use anyhow::{self, Error, Result};
pub use async_trait::async_trait;
pub use chrono::{DateTime, Local, Utc};
pub use futures::FutureExt;
pub use serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use serde_json::Value;
pub use std::{
	collections::*,
	env,
	fmt::{self, Debug, Display},
	fs,
	io::Write,
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
	macros::{self, *},
	// Verbose use/export to resolve name collisions with GRPC models.
	model::common::{Difficulty, Language},
	model::*,
	service::*,
	share::{share_prelude::*, *},
	ui::{config::*, theme::*, *},
	util::{time::*, *},
};

/// Unstable resources
/// Helps prevent name collisions internally as well.
///
pub use crate::{
	impls::{self, self as i},
	structs::{self, self as s, *},
	// Verbose use/export to resolve name collisions with external crates.
	traits::{self, self as t, Context, EventReceiver, *},
};

/// A non wasm32 target is non browser code, server, native, desktop
/// so this logic is safe for native/server/etc.
///
/// These conditionals will change to use all() & feature if we ever get around to mobile builds.
///
/// #[cfg(all(feature="native", not(target_arch = "wasm32")))]
///
#[cfg(not(target_arch = "wasm32"))]
pub use crate::{
	app::{context::*, *},
	modules::{
		native::{
			job,
			runtime::*,
			state::*,
			util::{logger::*, *},
			*,
		},
		sdlc::*,
		server::{self, channel, event::*, fs_deps::*},
	},
};

/// A Wasm32 build target
/// is code that runs client side (in browser) so we want this mod.
///
#[cfg(target_arch = "wasm32")]
pub use crate::modules::web::{app::*, bridge::*, *};
