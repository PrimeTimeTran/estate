use crate::model::resolver::crate_root;
use crate::{
	prelude::*,
	proto::{
		problem_service_client::ProblemServiceClient,
		submission_service_client::SubmissionServiceClient,
	},
};
use std::cmp::PartialEq;
use tray_icon::{Icon, TrayIconBuilder};

const TRAY_ICON_WIDTH: u32 = 16;
const TRAY_ICON_HEIGHT: u32 = 16;

impl<NativeCtx, S> ApplicationHandler<AppEvent> for Renderer<NativeCtx, S>
where
	NativeCtx: Ctx + 'static,
	S: Send + Sync + 'static,
{
	fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
		tracing::debug!("new_events")
	}
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		if self.settings.has_tray_icon == Some(true) {
			self.init_tray();
		}

		tracing::debug!("🔥 RESUMED");

		if self.windows.is_empty() {
			self.open_window(event_loop, crate::START_WINDOW);
		}
	}
	fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
		tracing::debug!("user_event");
		match event {
			AppEvent::Navigate(view) => {
				self.navigate_to(view);
			}

			AppEvent::RuntimeEvent => {
				tracing::debug!("user_event RuntimeEvent");
				let _ctx = self.app_context();
				Self::process_events(self);
				self.sync_views();
				// self.process_runtime_events();
				// self.window.request_redraw();
			}
			AppEvent::Shutdown => {
				tracing::debug!(">>> shutdown event received");
				tracing::debug!(">>> event_loop.exit() called");
			}
			AppEvent::ModifiersChanged {
				alt: _,
				command: _,
				ctrl: _,
				shift: _,
			} => {
				tracing::debug!("Modifiers Changed")
			}
			_ => {}
		}
	}
	fn window_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		tracing::debug!("window_event: {:?}", event);

		let Some(window) = self
			.windows
			.iter_mut()
			.find(|window| window.window.instance.id() == window_id)
		else {
			return;
		};

		let response = window
			.window
			.gui_state
			.on_window_event(&window.window.instance, &event);

		if response.repaint {
			window.window.instance.request_redraw();
		}

		match event {
			WindowEvent::Resized(size) => {
				window.window.resize(size);
				window.window.instance.request_redraw();
			}

			WindowEvent::RedrawRequested => {
				if window.window.occluded {
					return;
				}

				let mut ctx = AppContext {
					context: self.context.as_ref(),
					state: &mut self.state,
					event_tx: &mut self.event_tx,
					input: IOState::default(),
					last_revision: 0,
				};

				if let Err(e) = window.window.draw(&mut ctx) {
					tracing::error!("DEV >>> draw failed: {e:#}");
				}
			}

			_ => {}
		}
	}
	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: winit::event::DeviceId,
		_event: winit::event::DeviceEvent,
	) {
		tracing::debug!("device_event");
	}
	fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		tracing::debug!("about_to_wait");
		// self.app.update();
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		while let Ok(event) = MenuEvent::receiver().try_recv() {
			tracing::debug!("MenuEvent::receiver");
			println!("MenuEvent::receiver");
			self.handle_event(event, event_loop);
		}
	}
	fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
		tracing::debug!("suspended")
	}
	fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
		tracing::debug!("exiting")
	}

	fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
		tracing::debug!("memory_warning")
	}
}

impl<C, S> Renderer<C, S>
where
	C: Ctx,
	S: 'static,
{
	pub fn new(
		context: Arc<C>,
		state: S,
		settings: Arc<Settings>,
		cancel: CancellationToken,
		event_rx: C::EventReceiver,
		event_tx: C::EventSender,
	) -> Self {
		Self {
			event_rx,
			cancel,
			settings,
			state,
			event_tx,
			context,
			view: ViewType::MarkdownScreen,
			#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
			windows: vec![],

			#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
			menu_bar: None,
			#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
			tray_clock: None,
			#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
			tray_cursor: None,
		}
	}
	pub fn process_events(&mut self) {
		tracing::info!("renderer process_events");
		match self.event_rx.try_recv() {
			Some(event) => {
				let mut ctx = AppContext {
					context: self.context.as_ref(),
					state: &mut self.state,
					event_tx: &mut self.event_tx,
					// event_rx: &mut self.event_rx,
					input: IOState::default(),
					last_revision: 0,
				};

				#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
				tracing::info!("windows: {}", self.windows.len());

				#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
				for window in &mut self.windows {
					tracing::info!("dispatching event to screen");
					window.window.screen.event(&event, &mut ctx);
				}
			}

			None => {
				tracing::info!("RENDERER GOT NO EVENT");
			}
		}
	}
	pub fn _process_events(&mut self) {
		tracing::info!("renderer process_events");
		// loop {
		match self.event_rx.try_recv() {
			Some(event) => {
				tracing::info!(?event, "RENDERER GOT EVENT");

				let mut ctx = AppContext {
					context: self.context.as_ref(),
					state: &mut self.state,
					event_tx: &mut self.event_tx,
					// event_rx: &mut self.event_rx,
					input: IOState::default(),
					last_revision: 0,
				};

				#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
				tracing::info!("windows: {}", self.windows.len());
				#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
				for window in &mut self.windows {
					tracing::info!("dispatching event to screen");
					window.window.screen.event(&event, &mut ctx);
				}
			}

			None => {
				tracing::info!("RENDERER GOT NO EVENT");
				// break;
				// }
			}
		}
	}

	pub fn sync_views(&mut self) {
		tracing::info!("sync_views");
		#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
		for window in &mut self.windows {
			window.view = self.view;
			window.window.sync_view(window.view);
			window.window.instance.set_title(self.view.name());
			window.window.instance.request_redraw();
		}
	}
	pub fn navigate_to(&mut self, view: ViewType) {
		tracing::debug!("navigating from {:?} to {:?}", self.view, view,);
		self.view = view;
		self.sync_views();
	}
	pub fn app_context(&mut self) -> AppContext<'_, C, S> {
		AppContext {
			context: self.context.as_ref(),
			state: &mut self.state,
			// event_rx: &mut self.event_rx,
			event_tx: &mut self.event_tx,
			input: IOState::default(),
			last_revision: 0,
		}
	}
}

impl<NativeCtx, S> Renderer<NativeCtx, S>
where
	NativeCtx: Ctx + 'static,
	S: 'static,
{
	fn window_by_type(&mut self, kind: WindowType) -> Option<&mut AppWindow<NativeCtx, S>> {
		self.windows.iter_mut().find(|window| window.kind == kind)
	}
	fn open_window(&mut self, event_loop: &ActiveEventLoop, kind: WindowType) {
		tracing::debug!(" open window start");
		#[cfg(feature = "daemon")]
		{
			return;
		}
		if self.window_by_type(kind).is_some() {
			return;
		}
		match Window::new(event_loop, self.view) {
			Ok(window) => {
				tracing::debug!(" open window end, new window");
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
	fn handle_event(&mut self, _event: MenuEvent, _event_loop: &ActiveEventLoop) {
		tracing::debug!("handle_event");

		// match event {
		// 	MenuEvent::Navigate(view) => {
		// 		// self.navigate_to(view);
		// 	} // Other menu events...
		// 	  // MenuEvent::OpenWindow(kind) => {
		// 	  //   self.open_window(event_loop, kind);
		// 	  // }
		// }
	}

	// #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	fn _init_tray(&mut self) -> anyhow::Result<()> {
		if self.tray_cursor.is_some() {
			return Ok(());
		}

		let mut rgba = vec![0u8; (TRAY_ICON_WIDTH * TRAY_ICON_HEIGHT * 4) as usize];

		for y in 0..TRAY_ICON_HEIGHT {
			for x in 0..TRAY_ICON_WIDTH {
				let dx = x as f32 - 7.5;
				let dy = y as f32 - 7.5;

				if dx * dx + dy * dy <= 49.0 {
					let i = ((y * TRAY_ICON_WIDTH + x) * 4) as usize;
					rgba[i..i + 4].copy_from_slice(&[0, 0, 0, 255]);
				}
			}
		}

		let icon = Icon::from_rgba(rgba, TRAY_ICON_WIDTH, TRAY_ICON_HEIGHT)?;

		// let tray = TrayIconBuilder::new()
		// 	.with_icon(icon)
		// 	.with_icon_as_template(true)
		// 	.with_tooltip("Estate")
		// 	.build()?;

		let tray = TrayIconBuilder::new()
			.with_icon(icon)
			.with_icon_as_template(true)
			.with_tooltip("Estate")
			.build()
			.expect("failed to create tray icon");

		self.tray_cursor = Some(tray);
		Ok(())
	}

	fn init_tray(&mut self) -> anyhow::Result<()> {
		if self.tray_cursor.is_some() {
			return Ok(());
		}
		const TRAY_PNG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/estate-tray.png"));
		let image = image::load_from_memory(TRAY_PNG)?.into_rgba8();
		let (width, height) = image.dimensions();
		let icon = Icon::from_rgba(image.into_raw(), width, height)?;
		let tray = TrayIconBuilder::new()
			.with_icon(icon)
			.with_icon_as_template(true)
			.with_tooltip("Estate")
			.build()?;
		self.tray_cursor = Some(tray);
		Ok(())
	}
}
// impl Context {
// 	fn new(state: NativeState, api: ApiService) -> Self {
// 		Self { state, api }
// 	}
// }
// impl Default for Context {
// 	fn default() -> Self {
// 		Self::new(NativeState::default(), ApiService::default())
// 	}
// }

pub struct Renderer<C, S>
where
	C: Ctx,
{
	pub context: Arc<C>,
	pub state: S,
	pub view: ViewType,
	pub cancel: CancellationToken,
	pub event_rx: C::EventReceiver,
	pub event_tx: C::EventSender,
	pub settings: Arc<Settings>,

	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub windows: Vec<AppWindow<C, S>>,
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub menu_bar: Option<MenuBar>,
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub tray_clock: Option<MenuBar>,
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub tray_cursor: Option<TrayIcon>,
}
