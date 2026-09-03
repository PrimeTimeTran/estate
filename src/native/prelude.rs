//! # Description
//! Centralized internal dependency management for native platform targets like MacOS, Windows, Linux.
//!
//! 'pub use' enables external users of this crate to access the public dependencies.
pub use crate::{
	native::{
		self, constants_native::*, core::*, daemon::*, job::*, monitor::*, runtime::NativeRuntime,
		ui::*, window::*,
	},
	server::*,
};

/// # Description
/// Centralized external dependency management for native platform targets like MacOS, Windows, Linux.
///
/// pub enables downstream deps, "crate::native::*", to access the deps without importing again.
pub use cli::prelude::*;
pub use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
pub use signal_hook::{consts::SIGINT, iterator::Signals};
pub use tokio::{
	io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
	net::{TcpListener, UnixListener, UnixStream},
	runtime::Runtime as TokioRuntime,
	sync::{
		broadcast::{self, Receiver, Sender},
		mpsc::{self, UnboundedReceiver, UnboundedSender, channel, unbounded_channel},
		oneshot,
	},
};
pub use tokio_util::sync::CancellationToken;
