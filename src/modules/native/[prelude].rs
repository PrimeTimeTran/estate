//! # Description
//! Centralized internal dependency management for native platform targets like MacOS, Windows, Linux.
//!
//! 'pub use' enables external users of this crate to access the public dependencies.
//! The double prelude is done to manage native dependencies in a centralized manner, allowing for easier maintenance and updates.
//!

// ============================================================
// Native common
// ============================================================

pub use cli::context::*;
pub use cli::prelude::*;
pub use egui::MenuBar;
pub use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
pub use rmcp::{
	handler::server::wrapper::Parameters,
	model::{PromptMessage, PromptMessageContent},
};

pub use tray_icon::{
	Icon, TrayIcon, TrayIconBuilder,
	menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
};

pub use std::fs::OpenOptions;
pub use tokio::{
	io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
	net::TcpListener,
	runtime::Handle,
	sync::{
		broadcast::{self, Receiver, Sender},
		mpsc::{self, UnboundedReceiver, UnboundedSender, channel, unbounded_channel},
		oneshot,
	},
};

pub use ratatui::{Terminal, backend::CrosstermBackend};
pub use std::{io::stdout, time::Duration};
pub use tokio_util::sync::CancellationToken;
pub use tonic::{Request, Response, Status, transport::Channel};
pub use tracing::{debug, error, info, trace, warn};

// ============================================================
// Unix
// ============================================================

#[cfg(unix)]
pub use signal_hook::{consts::SIGINT, iterator::Signals};

#[cfg(unix)]
pub use tokio::net::{UnixListener, UnixStream};
//
// // ============================================================
// // macOS
// // ============================================================
//
// #[cfg(target_os = "macos")]
// pub use crate::native::macos::*;

/// # Description
/// Centralized external dependency management for native platform targets like MacOS, Windows, Linux.
///
/// pub enables downstream deps, "crate::native::*", to access the deps without importing again.
///
/// Warning: Do not remove items from here without running test suite passes without the removed items.
/// This is a central dependency management file for native platform targets like MacOS, Windows, Linux.
/// The items here are necessary to bring dependencies into scope for the native platform targets. Removing items may
/// cause compilation errors or runtime issues in the native platform targets.
//
// #[cfg(target_os = "windows")]
// pub use crate::native::platform;
pub use crate::{
	native::{
		app::*, daemon::*, job::*, monitor::*, resolver::*, screens::*, state, ui::*, window::*,
	},
	server::*,
};

pub use cli::prelude::*;
