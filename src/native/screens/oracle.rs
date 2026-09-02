use crate::{app::*, e, native::prelude::*, prelude::*, ui::Layout};

pub struct OracleScreen {
	active_focus: FocusedPane,
	dirty: bool,
	error: Option<String>,
	gesture: GestureController,
	last_direction: String,
	last_loaded: Option<SystemTime>,
	scroll_x: f32,
	scroll_y: f32,
}
impl<R: Runtime> Screen<R> for OracleScreen {
	fn configure(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {
		// Configure the regions this screen uses.
	}
	fn update(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {}
	fn event(&mut self, event: &e::Event, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>) {}
}
impl OracleScreen {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, NativeRuntime>) {
		self.draw_ui(ui, ctx);
		self.draw_status_bar(ui);
	}
	pub fn new() -> Self {
		Self {
			active_focus: FocusedPane::MainEditor,
			dirty: false,
			error: None,
			last_direction: String::new(),
			last_loaded: None,
			scroll_x: 0.0,
			scroll_y: 0.0,
			gesture: GestureController::new(),
		}
	}
	// -------------------------------------------------------------------------
	// INPUT
	// -------------------------------------------------------------------------
	fn inspect_trackpad(
		&mut self,
		ui: &mut egui::Ui,
		ctx: &AppContext<'_, NativeRuntime>,
	) -> TrackpadState {
		self.gesture.inspect(ui, &ctx.input)
	}
	// -------------------------------------------------------------------------
	// UI
	// -------------------------------------------------------------------------
	fn draw_ui(&mut self, ui: &mut egui::Ui, ctx: &AppContext<'_, NativeRuntime>) {
		self.draw_header(ui);

		let trackpad = self.inspect_trackpad(ui, ctx);

		self.draw_telemetry(ui, &trackpad);
		self.draw_trigger_preview(ui, &trackpad);
		self.draw_controls(ui);
	}
	fn draw_header(&mut self, ui: &mut egui::Ui) {
		ui.heading("Trackpad & Gesture Telemetry PoC");

		ui.label(
			"Goal: Inspect raw multi-axis vectors, modifiers, cursor position, hover target, and focus.",
		);

		ui.separator();
	}
	// -------------------------------------------------------------------------
	// TELEMETRY
	// -------------------------------------------------------------------------
	fn draw_telemetry(&mut self, ui: &mut egui::Ui, trackpad: &TrackpadState) {
		ui.columns(2, |columns| {
			// =================================================================
			// INPUT
			// =================================================================

			columns[0].group(|ui| {
				ui.heading("Input");

				ui.add_space(4.0);

				ui.label(format!("Scroll Delta X: {:.2}", trackpad.delta.x));

				ui.label(format!("Scroll Delta Y: {:.2}", trackpad.delta.y));

				ui.label(format!("Primary Axis: {}", trackpad.primary_axis()));

				ui.label(format!("Direction: {:?}", trackpad.direction));

				ui.add_space(8.0);

				Self::draw_modifier(ui, "Shift", trackpad.shift_held);
				Self::draw_modifier(ui, "Ctrl", trackpad.ctrl_held);
				Self::draw_modifier(ui, "Alt", trackpad.alt_held);
				Self::draw_modifier(ui, "Command", trackpad.command_held);
			});

			// =================================================================
			// CURSOR / FOCUS
			// =================================================================

			columns[1].group(|ui| {
				ui.heading("Cursor & Focus");

				ui.add_space(4.0);

				// -------------------------------------------------------------
				// Cursor position
				// -------------------------------------------------------------

				match trackpad.mouse_pos {
					Some(pos) => {
						ui.label(format!("Cursor X: {:.1}", pos.x));
						ui.label(format!("Cursor Y: {:.1}", pos.y));
						ui.label(format!("Cursor: ({:.1}, {:.1})", pos.x, pos.y));
					}

					None => {
						ui.label("Cursor: Outside viewport");
					}
				}

				ui.add_space(8.0);

				// -------------------------------------------------------------
				// Cursor target
				// -------------------------------------------------------------

				ui.horizontal(|ui| {
					ui.label("Hovered:");

					let hovered = trackpad.hovered != CursorTarget::None;

					ui.colored_label(
						if hovered {
							egui::Color32::LIGHT_GREEN
						} else {
							egui::Color32::GRAY
						},
						trackpad.hovered_name(),
					);
				});

				// -------------------------------------------------------------
				// Focus
				// -------------------------------------------------------------

				ui.horizontal(|ui| {
					ui.label("Focus:");

					ui.colored_label(egui::Color32::LIGHT_BLUE, format!("{:?}", trackpad.focus));
				});

				ui.add_space(8.0);

				// -------------------------------------------------------------
				// Gesture state
				// -------------------------------------------------------------

				ui.label(format!(
					"Side Panel Width: {:.1}px",
					self.gesture.state.side_panel_width
				));

				ui.label(format!(
					"Secondary Scroll: {:.1}",
					self.gesture.state.secondary_scroll_offset
				));
			});
		});

		ui.add_space(12.0);
		ui.separator();
	}
	fn draw_modifier(ui: &mut egui::Ui, name: &str, held: bool) {
		ui.horizontal(|ui| {
			ui.label(format!("{name}:"));

			ui.colored_label(
				if held {
					egui::Color32::LIGHT_GREEN
				} else {
					egui::Color32::GRAY
				},
				if held { "HELD" } else { "Released" },
			);
		});
	}
	// -------------------------------------------------------------------------
	// TRIGGER PREVIEW
	// -------------------------------------------------------------------------
	fn draw_trigger_preview(&mut self, ui: &mut egui::Ui, trackpad: &TrackpadState) {
		ui.group(|ui| {
			ui.heading("Target Action Trigger Preview");

			ui.add_space(4.0);

			// First verify the modifier independently of the gesture.
			if trackpad.shift_held {
				ui.colored_label(egui::Color32::LIGHT_GREEN, "SHIFT DETECTED");
			} else {
				ui.label("SHIFT NOT HELD");
			}

			ui.add_space(4.0);

			let is_horizontal = trackpad.delta.x.abs() > trackpad.delta.y.abs();

			let is_vertical = trackpad.delta.y.abs() > trackpad.delta.x.abs();

			if trackpad.shift_held && is_horizontal {
				ui.colored_label(
					egui::Color32::LIGHT_BLUE,
					format!(
						"⚡ TRIGGER MATCH: Resize Panel Vector -> {:.2}px",
						trackpad.delta.x
					),
				);
			} else if trackpad.shift_held && is_vertical {
				ui.colored_label(
					egui::Color32::LIGHT_GREEN,
					format!(
						"⚡ TRIGGER MATCH: Cross-Scroll Secondary Pane -> {:.2} units",
						trackpad.delta.y
					),
				);
			} else {
				ui.label("Waiting for scroll...");
			}
		});
	}
	fn draw_controls(&mut self, _ui: &mut egui::Ui) {}
	fn draw_status_bar(&mut self, ui: &mut egui::Ui) {
		ui.add_space(8.0);

		ui.horizontal(|ui| {
			if ui.button("Reset Telemetry States").clicked() {
				// Reset state here when you decide what "reset" means.
			}

			ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
				ui.label("PoC V1.0 - Ready for OS Daemon translation");
			});
		});
	}
}
