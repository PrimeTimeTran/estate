pub mod agent;
pub mod backend;
pub mod daemon;
pub mod discovery;
pub mod lint;
pub mod monitor;
pub mod native_job;
pub mod native_prelude;
pub mod poc;
pub mod resolver;
pub mod router;
pub mod screens;
pub mod scroll;
pub mod state;
pub mod task;
pub mod ui;
pub mod window;

pub use self::{discovery::*, native_prelude::*, scroll::*};
pub use screens::*;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

use crate::{
	doc,
	prelude::{traits::Ctx, *},
	ui::*,
	ui_prelude as gui,
};

impl<NativeCtx, S> ApplicationHandler<AppEvent> for app_entry::Renderer<NativeCtx, S>
where
	NativeCtx: Ctx,
	S: Send + Sync + 'static,
{
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("about_to_wait");
		// self.app.update();
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			tracing::info!("MenuEvent::receiver");
			println!("MenuEvent::receiver");
			self.handle_event(event, event_loop);
		}
	}

	fn device_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		device_id: winit::event::DeviceId,
		event: winit::event::DeviceEvent,
	) {
		tracing::info!("device_event");
	}

	fn exiting(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("exiting")
	}

	fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("memory_warning")
	}
	fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
		tracing::info!("new_events")
	}
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("🔥 RESUMED");
		if self.windows.is_empty() {
			self.open_window(event_loop, crate::START_WINDOW);
		}
		// self.device_event(event_loop, device_id, event);
		// if self.context().menu_bar.is_none() {
		// let menu = Self::menu_bar(true);
		// menu.init_for_nsapp();
		// self.menu_bar = Some(menu);
		// }
		// if self.context().windows.is_empty() {
		// 	// self.open_window(event_loop, crate::START_WINDOW);
		// }
		// if self.context().tray_clock.is_none() {
		// 	// let (menu, tray) = match Self::bootstrap() {
		// 	// 	Ok(value) => value,
		// 	// 	Err(error) => {
		// 	// 		tracing::error!(%error, "failed to bootstrap tray");
		// 	// 		return;
		// 	// 	}
		// 	// };
		// 	// self.menu = Some(menu);
		// 	// self.tray_clock = Some(tray);
		// 	tracing::debug!("🔥 main tray initialized");
		// }
		// if self.context().tray_cursor.is_none() {
		// 	match TrayIconBuilder::new()
		// 		.with_icon(scroll_tray_icon())
		// 		.with_tooltip("Estate Scroll Controller")
		// 		.build()
		// 	{
		// 		Ok(tray) => {
		// 			self.context().tray_cursor = Some(tray);
		// 			tracing::debug!("🔥 scroll tray initialized");
		// 		}
		// 		Err(error) => {
		// 			tracing::error!(%error, "failed to create scroll tray");
		// 		}
		// 	}
		// }
	}

	fn suspended(&mut self, event_loop: &ActiveEventLoop) {
		tracing::info!("suspended")
	}
	fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
		tracing::info!("user_event");
		println!("user_event");
		match event {
			AppEvent::RuntimeEvent => {
				println!("user_event RuntimeEvent");
				tracing::info!("user_event RuntimeEvent");
				// tracing::info!("user_event");

				// self.app.update();
				// self.sync_views();
			}
			// 	AppEvent::Navigate(view) => {
			// 		// self.host.run();
			// 		// self.host.worker().
			// 		self
			// 			.host
			// 			.worker()
			// 			.runtime
			// 			.spawn(e::Event::app(e::Klass::Navigate(view)));
			// 		// self.runtime().emit(e::Event::app(e::Klass::Navigate(view)));
			// 		// self.app.update();
			// 		// self.sync_views();
			// 	}
			AppEvent::Shutdown => {
				tracing::debug!(">>> shutdown event received");
				// self.shutdown();

				tracing::debug!(">>> event_loop.exit() called");
			}
			// 	AppEvent::CursorPosition { x, y } => {
			// 		// let text = format!("↖ {:.0}  {:.0}", x, y);
			// 		// let text = format!("← {:.0}  {:.0}", x, y);
			// 		// let text = format!("→ {:.0}  {:.0}", x, y);
			// 		// let text = format!("↑ {:.0}  {:.0}", x, y);
			// 		// let text = format!("● {:.0}, {:.0}", x, y);
			// 		// let text = format!("◉ {:.0}, {:.0}", x, y);
			// 		let text = format!("⌖ {:.0}, {:.0}", x, y);
			// 		// let text = format!("🟢 {:.0}, {:.0}", x, y);
			// 		// let text = format!("🔵 {:.0}, {:.0}", x, y);
			// 		// let text = format!("🟡 {:.0}, {:.0}", x, y);
			// 		// let text = format!("🔴 {:.0}, {:.0}", x, y);
			// 		// let region = if x < 960.0 { "← LEFT" } else { "RIGHT →" };
			// 		if let Some(tray) = &self.tray_cursor {
			// 			let _ = tray.set_title(Some(text));
			// 		}
			// 	}
			// 	AppEvent::TickClock(text) => {
			// 		if let Some(tray) = &self.context().tray_clock {
			// 			// let _ = tray.set_title(Some(text));
			// 		}
			// 		// self.sync_views();
			// 	}
			AppEvent::ModifiersChanged {
				alt,
				command,
				ctrl,
				shift,
			} => {
				tracing::info!("Modifiers Changed")
			}
			_ => {}
		}
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		tracing::info!("window_event");
		// let Some(window) = self.state() else {
		// 	return;
		// };
		// let response = window
		// 	.window
		// 	.gui_state
		// 	.on_window_event(&window.window.instance, &event);
		// if response.repaint {
		// 	window.window.instance.request_redraw();
		// }
		match event {
			// WindowEvent::CloseRequested => {
			// 	tracing::debug!("🛑 Window close requested for id: {:?}", window_id);
			// 	self
			// 		.windows
			// 		.retain(|window| window.window.instance.id() != window_id);
			// 	return;
			// }
			// WindowEvent::RedrawRequested => {
			// 	if window.window.occluded {
			// 		return;
			// 	}
			// 	let menu = {
			// 		let event_rx = self.app.engine.runtime().subscribe();
			// 		let mut ctx = AppContext {
			// 			app: &mut self.app,
			// 			input: IOState::default(),
			// 			event_rx,
			// 			last_revision: 0,
			// 		};
			// 		if let Err(e) = window.window.draw(&mut ctx) {
			// 			tracing::error!("DEV >>> draw failed: {e:#}");
			// 		}
			// 	};
			// }
			// WindowEvent::Focused(true) => {
			// 	window.window.instance.request_redraw();
			// }
			// WindowEvent::Occluded(occluded) => {
			// 	window.window.occluded = occluded;
			// 	if !occluded {
			// 		window.window.instance.request_redraw();
			// 	}
			// }
			// WindowEvent::Resized(size) => {
			// 	if size.width == 0 || size.height == 0 {
			// 		return;
			// 	}
			// 	// window.window.config.width = size.width;
			// 	// window.window.config.height = size.height;
			// 	// window
			// 	// 	.window
			// 	// 	.surface
			// 	// 	.configure(&window.window.device, &window.window.config);
			// 	// window.window.needs_resize = false;
			// 	// window.window.instance.request_redraw();
			// }
			_ => {}
		}
	}
}

impl<NativeCtx, S> app_entry::Renderer<NativeCtx, S> {
	fn window_by_type(&mut self, kind: WindowType) -> Option<&mut AppWindow> {
		self.windows.iter_mut().find(|window| window.kind == kind)
	}
	fn open_window(&mut self, event_loop: &ActiveEventLoop, kind: WindowType) {
		tracing::info!(" open window start");
		if self.window_by_type(kind).is_some() {
			return;
		}
		match Window::new(event_loop, self.view) {
			Ok(window) => {
				tracing::info!(" open window end, new window");
				window.instance.set_title(self.view.name().into());
				self.windows.push(AppWindow {
					// runtime: self.runtime.clone(),
					kind,
					view: self.view,
					window,
				});
			}
			Err(error) => {
				tracing::error!("failed to create window: {error}");
			}
		}
	}
	fn handle_event(&mut self, event: MenuEvent, event_loop: &ActiveEventLoop) {
		tracing::info!("handle_event");
	}
}

#[derive(Debug, Clone)]
pub struct CursorDaemon<S> {
	pub sink: S,
	pub cancel: CancellationToken,
}
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
	pub x: f64,
	pub y: f64,
}
