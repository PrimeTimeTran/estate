use crate::prelude::*;

pub use anyhow::{Context as CtxAnyhow, Result};
pub use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes, kCFRunLoopDefaultMode};
pub use core_graphics::{
	display::{CGDisplay, CGPoint, CGRect},
	event::*,
	event_source::{CGEventSource, CGEventSourceStateID},
	geometry,
};
pub use mach2::mach_time;
pub use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

pub mod app_macos;
pub use app_macos::*;

pub use host::*;
mod host;

pub mod keymap;
pub use keymap::*;

pub mod hid;
pub use hid::*;

pub mod scroll;
pub use scroll::*;

pub mod window;
pub use window::*;

pub mod server;
pub use server::*;

impl Context {
	fn new(state: NativeState, api: ApiService) -> Self {
		Self { state, api }
	}
}
impl Ctx for Context {
	fn api(&self) -> &Self::Api {
		&self.api
	}
	fn api_mut(&mut self) -> &mut Self::Api {
		&mut self.api
	}
	fn initial_state() -> Self::AppState {
		structs::S {
			context: PhantomData,
			state: PhantomData,
			view: ViewType::MarkdownScreen,
		}
	}
	type Api = ApiService;
	type AppState = structs::S<Context>;
	type EventReceiver = structs::BroadcastReceiver<e::Event>;
	type EventSender = structs::BroadcastSender<e::Event>;
	type GuiState = NativeGuiState;
}
impl Default for Context {
	fn default() -> Self {
		Self::new(NativeState::default(), ApiService::default())
	}
}

#[derive(Clone)]
pub struct Context {
	pub state: NativeState,
	pub api: ApiService,
}
