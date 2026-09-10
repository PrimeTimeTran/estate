pub use anyhow::{self, Error, Result};
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use futures::FutureExt;
pub use serde::{Deserialize, Serialize, de::DeserializeOwned};
pub use serde_json::Value;
pub use tokio::task::JoinHandle;

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
	app::{
		app_entry::{self, Renderer, *},
		app_state, *,
	},
	app_macros,
	app_state::*,
	data::*,
	e,
	r#macro::*,
	model::*,
	runtime::*,
	services::*,
	share::{share_prelude::*, *},
	tool::{time::*, *},
	r#trait as traits,
	r#trait::{Context, EventReceiver, *},
	ui::{config::*, theme::*, ui_prelude::*, ui_trait::*, *},
};

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub use crate::{app::app_web::*, web::*};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use crate::{
	app::{app_native::*, context::*, *},
	logger::*,
	native::native_prelude::*,
	native_state::*,
	server::{self, events::*},
};

pub mod enums {}
pub mod impls {
	use super::structs::*;
	use crate::prelude::*;

	impl<C> Clone for S<C> {
		fn clone(&self) -> Self {
			Self {
				context: PhantomData,
				state: PhantomData,
				view: self.view.clone(),
			}
		}
	}
	/// Manually implement Default specifically for S<C>
	///
	impl Default for S<C> {
		fn default() -> Self {
			S {
				view: ViewType::MarkdownScreen,
				context: PhantomData,
				state: PhantomData,
			}
		}
	}
}

pub mod structs {
	use super::impls::*;
	use crate::prelude::*;
	pub struct Linux;
	pub struct MacOS;
	pub struct Windows;

	pub use crate::app_entry::Renderer;

	pub struct C;

	/// State vs Context is like "Nature vs Nurture",there is no perfect answer to what drives what.
	/// Every state depends on some context which depending on how you think of it, might be considered "state" as well.
	///
	/// So for now, in order to implement a Type State system robustly, we're going to agree that all apps/processes must come from a context.
	///
	/// Linux, MacOS, Windows, they're all contexts in which the app can run so we begin our app with that assumption for modeling more robustly.
	///
	#[derive(Debug)]
	pub struct S<C> {
		pub context: PhantomData<C>,
		pub state: PhantomData<C>,
		pub view: ViewType,
	}
}

pub use crate::prelude::{impls as i, structs as s};
