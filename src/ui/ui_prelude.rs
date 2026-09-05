// This looks like its an uncommented file, but it is actually a prelude file for the UI module.
// It is used to bring all the necessary dependencies into scope for the UI module.
// The prelude file is used to manage dependencies in a centralized manner,
// allowing for easier maintenance and updates.
//
pub use egui::{
	Align, ClippedPrimitive, Color32, Context, Direction, Frame, FullOutput, Id, Margin, ScrollArea,
	TexturesDelta, Ui, UiBuilder, ViewportId, containers::Panel as EguiPanel,
};
pub use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
pub use egui_extras::{Column, TableBuilder};
pub use egui_plot::{Bar, BarChart, Line, Plot, Points};
pub use egui_wgpu::{Renderer, RendererOptions, wgpu};
pub use global_hotkey::{
	GlobalHotKeyEvent, GlobalHotKeyManager,
	hotkey::{Code, HotKey, Modifiers},
};
pub use std::fmt;
pub use strum::IntoStaticStr;
pub use wgpu::{Adapter, Device, SurfaceColorSpace};
pub use winit::{
	dpi::{PhysicalPosition, PhysicalSize},
	event_loop::ActiveEventLoop,
};

pub use crate::{
	panel::*,
	prelude::*,
	proto::{
		types::{self, *},
		*,
	},
	r#trait::Runtime,
	ui::{
		config::*, layout::*, primitive::*, region::*, screen::*, theme::palette::*, ui_trait::*,
		view::*, *,
	},
};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use cli::context::*;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use tokio::sync::mpsc;
