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
	logger::*,
	monitor::*,
	poc::*,
	resolver::*,
	runtime::{NativeRuntime, *},
	state::*,
	ui::*,
	ve::*,
	window::*,
};

/// # Description
/// Centralized external dependency management for native platform targets like MacOS, Windows, Linux.
///
pub use cli::prelude::*;
pub use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
pub use signal_hook::{consts::SIGINT, iterator::Signals, *};
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
