//! # Description
//! Centralized internal dependency management for native platform targets like MacOS, Windows, Linux.

pub use crate::native::{
	self,
	app::*,
	backend::*,
	constants_native::*,
	core::*,
	daemon::*,
	job::*,
	monitor::*,
	// native::{self, prelude::*, resolver::engine_data_dir},
	poc::*,
	resolver::*,
	router::*,
	runtime::{NativeRuntime, *},
	state::*,
	ui::*,
	ve::*,
	window::*,
	windows::*,
};

pub use crate::event::*;
pub use crate::logger::*;

pub use crate::native::linux::*;
pub use crate::native::macos::*;
pub use crate::native::runtime::*;

/// # Description
/// Centralized external dependency management for native platform targets like MacOS, Windows, Linux.
///
pub use cli::{self, context::*, prelude::*, *};
pub use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
pub use signal_hook::{self, *};
pub use tokio::{
	io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
	net::{TcpListener, UnixListener, UnixStream},
	runtime::Runtime as TokioRuntime,
	sync::{
		broadcast::{self, Receiver, Sender},
		mpsc::{self, UnboundedSender, channel},
		oneshot,
	},
};
