//! # Description
//! Centralized internal dependency management for native platform targets like MacOS, Windows, Linux.
//!
//! 'pub use' enables external users of this crate to access the public dependencies.
//! The double prelude is done to manage native dependencies in a centralized manner, allowing for easier maintenance and updates.
//!
pub use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
pub use core_graphics::{
	display::{CGDisplay, CGPoint, CGRect},
	event::*,
	event_source::{CGEventSource, CGEventSourceStateID},
	geometry,
};

pub use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
pub use rmcp::{
	handler::server::wrapper::Parameters,
	model::{PromptMessage, PromptMessageContent},
};
pub use signal_hook::{consts::SIGINT, iterator::Signals};
pub use tokio::{
	io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
	net::{TcpListener, UnixListener, UnixStream},
	runtime::Handle,
	sync::{
		broadcast::{self, Receiver, Sender},
		mpsc::{self, UnboundedReceiver, UnboundedSender, channel, unbounded_channel},
		oneshot,
	},
};
pub use tokio_util::sync::CancellationToken;
pub use tonic::transport::Channel;
pub use tray_icon::{
	Icon, TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
};
// pub use winit::event_loop::EventLoopProxy;
pub use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS},
	window::WindowId,
};

/// # Description
/// Centralized external dependency management for native platform targets like MacOS, Windows, Linux.
///
/// pub enables downstream deps, "crate::native::*", to access the deps without importing again.
///
/// Warning: Do not remove items from here without running test suite passes without the removed items.
/// This is a central dependency management file for native platform targets like MacOS, Windows, Linux.
/// The items here are necessary to bring dependencies into scope for the native platform targets. Removing items may
/// cause compilation errors or runtime issues in the native platform targets.
pub use crate::native::state as native_state;
pub use crate::{
	native::{self, daemon::*, lint::*, monitor::*, native_job::*, ui::*, window::*},
	// runtime::*,
	server::*,
};

pub use cli::prelude::*;
