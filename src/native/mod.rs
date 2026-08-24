#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

pub mod app;
pub mod constants;
pub mod prelude;
pub mod router;
pub mod ui;
pub mod ve;
pub mod window;

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

// pub use uuid::{Uuid, *};
