//! # Description
//! Centralized internal dependency management for native platform targets like MacOS, Windows, Linux.
//!
pub use crate::native::{
	self, app::*, backend::*, constants_native::*, core::*, daemon::*, job::*, monitor::*, poc::*,
	router::*, runtime::*, ui::*, ve::*, window::*, windows::*,
};

pub use crate::native::linux::*;
pub use crate::native::macos::*;

/// # Description
/// Centralized external dependency management for native platform targets like MacOS, Windows, Linux.
///
pub use cli::{self, context::*, prelude::*, *};
pub use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
pub use signal_hook::{self, *};
pub use tokio::{
	io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
	net::{TcpListener, UnixListener, UnixStream},
	runtime::Runtime,
	sync::{
		broadcast::{self, Receiver, Sender},
		mpsc::{self, UnboundedSender, channel},
		oneshot,
	},
};
