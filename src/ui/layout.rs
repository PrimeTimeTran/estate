use crate::{e, prelude::*, region::Panel};

use crate::LAYOUT as config;

pub struct Layout<R: Runtime, E: Executor> {
	pub activity_bar: Panel<R, E>,
	pub dock_left: Panel<R, E>,
	pub main: Panel<R, E>,
	pub primary_bar: Panel<R, E>,
	pub secondary_bar: Panel<R, E>,
	pub bottom_panel: Panel<R, E>,
	pub status_bar: Panel<R, E>,
	pub dock_right: Panel<R, E>,
}
impl<R: Runtime, E: Executor> Layout<R, E> {
	// Rust uses ownership,borrowing, and lifetimes to determine when values
	// may be safely destroyed, allowing memory to be reclaimed deterministically
	// without a garbage collector.
	pub fn new() -> Self {
		let main = ProblemView::new();
		let dock_left = ProblemViewSidebar::new();
		let bottom_panel = ProblemViewBottomPanel::new();
		Self {
			main: Panel::from_config(main, Region::content(), &PanelState::new(true, 0.0)),
			activity_bar: Panel::from_config(
				DebugPanel::new("Activity Bar"),
				Region::fixed(config.activity_bar.effective_size()),
				&config.activity_bar,
			),
			dock_left: Panel::from_config(
				dock_left,
				config.dock_left.region(50.0, 600.0).with_fill(config.bg),
				&config.dock_left,
			),
			primary_bar: Panel::from_config(
				DebugPanel::new("Primary Bar"),
				config.primary_bar.region(50.0, 600.0).with_fill(config.bg),
				&config.primary_bar,
			),
			secondary_bar: Panel::from_config(
				DebugPanel::new("Secondary Bar"),
				config
					.secondary_bar
					.region(50.0, 600.0)
					.with_fill(config.bg),
				&config.secondary_bar,
			),
			bottom_panel: Panel::from_config(
				bottom_panel,
				config.bottom_panel.region(50.0, 600.0).with_fill(config.bg),
				&config.bottom_panel,
			),
			status_bar: Panel::from_config(
				DebugPanel::new("Status Bar"),
				Region::fixed(config.status_bar.effective_size())
					.with_fill(config.bg)
					.with_top_border(true),
				&config.status_bar,
			),
			dock_right: Panel::from_config(
				DebugPanel::new("Dock Right"),
				config.dock_right.region(50.0, 600.0).with_fill(config.bg),
				&config.dock_right,
			),
		}
	}
}
impl<R: Runtime, E: Executor> LayoutTrait<R, E> for Layout<R, E> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R, E>) {
		let rect = ui.max_rect();
		ui.painter()
			.rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 30));
		ui.painter().rect_stroke(
			rect,
			0.0,
			egui::Stroke::new(5.0, egui::Color32::RED),
			egui::StrokeKind::Outside,
		);
		let mouse_pos = ui.input(|i| i.pointer.hover_pos());
		let available = ui.available_rect_before_wrap();
		// Background
		ui.painter().rect_filled(available, 0.0, LAYOUT.bg);
		// Calculate all region boundaries from the current region sizes.
		let layout = self.calculate_region_boundaries(available);
		// Determine what region the cursor is currently over.
		let cursor_target = self.cursor_target(mouse_pos, &layout);
		// Update application input state.

		// self.update_input(ctx, ui, mouse_pos);
		ctx.input = IOState {
			cursor_pos: mouse_pos,
			cursor_target,
			shift_held: ui.input(|i| i.modifiers.shift),
			ctrl_held: ui.input(|i| i.modifiers.ctrl),
			alt_held: ui.input(|i| i.modifiers.alt),
			command_held: ui.input(|i| i.modifiers.command),
			primary_down: ui.input(|i| i.pointer.primary_down()),
		};
		// ---------------------------------------------------------
		// Draw panels
		// ---------------------------------------------------------
		Self::draw_panel(ui, ctx, layout.activity_bar, &mut self.activity_bar);
		if self.dock_left.open {
			Self::draw_panel(ui, ctx, layout.dock_left, &mut self.dock_left);
		}
		Self::draw_panel(ui, ctx, layout.primary_bar, &mut self.primary_bar);
		Self::draw_panel(ui, ctx, layout.secondary_bar, &mut self.secondary_bar);
		Self::draw_panel(ui, ctx, layout.main, &mut self.main);
		if self.bottom_panel.open {
			Self::draw_panel(ui, ctx, layout.bottom_panel, &mut self.bottom_panel);
		}
		if self.dock_right.open {
			Self::draw_panel(ui, ctx, layout.dock_right, &mut self.dock_right);
		}
		Self::draw_panel(ui, ctx, layout.status_bar, &mut self.status_bar);
		// ---------------------------------------------------------
		// Draw resize handles LAST
		// ---------------------------------------------------------
		if self.dock_left.open {
			Self::resize_region(
				ui,
				"dock_left_resize",
				layout.dock_left,
				&mut self.dock_left.region,
				ResizeEdge::Right,
				1.0,
			);
		}
		if self.bottom_panel.open {
			Self::resize_region(
				ui,
				"bottom_panel_resize",
				layout.bottom_panel,
				&mut self.bottom_panel.region,
				ResizeEdge::Top,
				-1.0,
			);
		}
		if self.dock_right.open {
			Self::resize_region(
				ui,
				"dock_right_resize",
				layout.dock_right,
				&mut self.dock_right.region,
				ResizeEdge::Left,
				-1.0,
			);
		}
	}
	fn update(&mut self, _ctx: &mut AppContext<'_, R, E>) {}
	fn event(&mut self, _event: &e::Event, _ctx: &mut AppContext<'_, R, E>) {}
}
impl<R: Runtime, E: Executor> Layout<R, E> {
	fn draw_view(
		ui: &mut egui::Ui,
		rect: egui::Rect,
		view: &mut dyn ViewTrait<R, E>,
		ctx: &mut AppContext<'_, R, E>,
	) {
		while let Some(event) = ctx.next_event() {
			view.event(&event, ctx);
		}
		let mut child = ui.new_child(
			egui::UiBuilder::new()
				.max_rect(rect)
				.layout(egui::Layout::top_down(egui::Align::LEFT)),
		);
		view.draw(&mut child, ctx);
	}
	fn draw_panel(
		ui: &mut egui::Ui,
		ctx: &mut AppContext<'_, R, E>,
		rect: egui::Rect,
		panel: &mut Panel<R, E>,
	) {
		if !panel.open {
			return;
		}
		// Outer panel appearance.
		if let Some(fill) = panel.region.fill {
			ui.painter().rect_filled(rect, 0.0, fill);
		}
		if panel.region.top_border {
			ui.painter().line_segment(
				[rect.left_top(), rect.right_top()],
				egui::Stroke::new(1.0, palette::BORDER),
			);
		}
		// Inner content area.
		let content_rect = panel.region.content_rect(rect);
		Self::draw_view(ui, content_rect, panel.content.as_mut(), ctx);
	}
	fn calculate_region_boundaries(&mut self, available: egui::Rect) -> VeLayout {
		// =========================================================
		// Fixed outer regions
		// =========================================================
		let activity_bar_width = if self.activity_bar.open {
			self.activity_bar.region.size
		} else {
			0.0
		};
		let status_bar_height = if self.status_bar.open {
			self.status_bar.region.size
		} else {
			0.0
		};
		let activity_bar = egui::Rect::from_min_max(
			available.min,
			egui::pos2(available.left() + activity_bar_width, available.bottom()),
		);
		let workspace_rect = egui::Rect::from_min_max(
			egui::pos2(available.left() + activity_bar_width, available.top()),
			egui::pos2(available.right(), available.bottom() - status_bar_height),
		);
		let status_bar = egui::Rect::from_min_max(
			egui::pos2(available.left(), workspace_rect.bottom()),
			available.max,
		);
		// =========================================================
		// Workspace: left / center / right
		// =========================================================
		let min_main_width = 100.0;
		let requested_left = if self.dock_left.open {
			self.dock_left.region.size
		} else {
			0.0
		};
		let requested_right = if self.dock_right.open {
			self.dock_right.region.size
		} else {
			0.0
		};
		let available_side_width = (workspace_rect.width() - min_main_width).max(0.0);
		let requested_total = requested_left + requested_right;
		let scale = if requested_total > available_side_width {
			available_side_width / requested_total
		} else {
			1.0
		};
		let left_width = requested_left * scale;
		let right_width = requested_right * scale;
		let dock_left = egui::Rect::from_min_max(
			workspace_rect.min,
			egui::pos2(workspace_rect.left() + left_width, workspace_rect.bottom()),
		);
		let dock_right = egui::Rect::from_min_max(
			egui::pos2(workspace_rect.right() - right_width, workspace_rect.top()),
			workspace_rect.max,
		);
		let center_rect = egui::Rect::from_min_max(
			egui::pos2(workspace_rect.left() + left_width, workspace_rect.top()),
			egui::pos2(
				workspace_rect.right() - right_width,
				workspace_rect.bottom(),
			),
		);
		// =========================================================
		// CENTER: tabs / breadcrumbs / main / bottom
		// =========================================================
		let min_main_height = 100.0;
		// =========================================================
		// Tabs
		// =========================================================
		let tabs_height = if self.primary_bar.open {
			self.primary_bar.region.size
		} else {
			0.0
		};
		let breadcrumbs_height = if self.secondary_bar.open {
			self.secondary_bar.region.size
		} else {
			0.0
		};
		let primary_bar = egui::Rect::from_min_max(
			center_rect.min,
			egui::pos2(center_rect.right(), center_rect.top() + tabs_height),
		);
		// =========================================================
		// Breadcrumbs
		// =========================================================
		let secondary_bar = egui::Rect::from_min_max(
			egui::pos2(center_rect.left(), primary_bar.bottom()),
			egui::pos2(
				center_rect.right(),
				primary_bar.bottom() + breadcrumbs_height,
			),
		);
		// =========================================================
		// Center content
		// =========================================================
		let content_rect = egui::Rect::from_min_max(
			egui::pos2(center_rect.left(), secondary_bar.bottom()),
			center_rect.max,
		);
		// =========================================================
		// Main / bottom
		// =========================================================
		let bottom_height = if self.bottom_panel.open {
			self
				.bottom_panel
				.region
				.size
				.min((content_rect.height() - min_main_height).max(0.0))
		} else {
			0.0
		};
		let main = egui::Rect::from_min_max(
			content_rect.min,
			egui::pos2(content_rect.right(), content_rect.bottom() - bottom_height),
		);
		let bottom_panel = egui::Rect::from_min_max(
			egui::pos2(content_rect.left(), content_rect.bottom() - bottom_height),
			content_rect.max,
		);
		VeLayout {
			activity_bar,
			bottom_panel,
			dock_left,
			dock_right,
			main,
			primary_bar,
			secondary_bar,
			status_bar,
		}
	}
	fn resize_handle(
		ui: &mut egui::Ui,
		id: &str,
		rect: egui::Rect,
		edge: ResizeEdge,
		mut resize: impl FnMut(f32),
	) {
		let cursor = match edge {
			ResizeEdge::Left | ResizeEdge::Right => egui::CursorIcon::ResizeHorizontal,
			ResizeEdge::Top | ResizeEdge::Bottom => egui::CursorIcon::ResizeVertical,
		};
		let response = ui.interact(rect, egui::Id::new(id), egui::Sense::drag());
		if response.hovered() || response.dragged() {
			ui.ctx().set_cursor_icon(cursor);
		}
		// Visible divider.
		let divider = match edge {
			ResizeEdge::Left | ResizeEdge::Right => egui::Rect::from_min_max(
				egui::pos2(rect.center().x - 1.0, rect.top()),
				egui::pos2(rect.center().x + 1.0, rect.bottom()),
			),
			ResizeEdge::Top | ResizeEdge::Bottom => egui::Rect::from_min_max(
				egui::pos2(rect.left(), rect.center().y - 1.0),
				egui::pos2(rect.right(), rect.center().y + 1.0),
			),
		};
		ui.painter().rect_filled(divider, 0.0, palette::BORDER);
		if response.dragged() {
			let delta = match edge {
				ResizeEdge::Left | ResizeEdge::Right => response.drag_motion().x,
				ResizeEdge::Top | ResizeEdge::Bottom => response.drag_motion().y,
			};
			resize(delta);
		}
	}
	fn resize_handle2(ui: &mut egui::Ui, id: &str, rect: egui::Rect, mut resize: impl FnMut(f32)) {
		let cursor = if id == "bottom_panel_resize" {
			egui::CursorIcon::ResizeVertical
		} else {
			egui::CursorIcon::ResizeHorizontal
		};
		let response = ui.interact(rect, egui::Id::new(id), egui::Sense::drag());
		if response.hovered() || response.dragged() {
			ui.ctx().set_cursor_icon(cursor);
		}
		// Visible resize divider.
		let divider = match cursor {
			egui::CursorIcon::ResizeVertical => {
				egui::Rect::from_center_size(rect.center(), egui::vec2(2.0, rect.height()))
			}
			_ => egui::Rect::from_center_size(rect.center(), egui::vec2(rect.width(), 2.0)),
		};
		let color = if response.hovered() || response.dragged() {
			palette::BORDER
		} else {
			palette::BORDER
		};
		ui.painter().rect_filled(divider, 0.0, color);
		if response.dragged() {
			let delta = match cursor {
				egui::CursorIcon::ResizeVertical => response.drag_motion().y,
				_ => response.drag_motion().x,
			};
			resize(delta);
		}
	}
	fn resize_region(
		ui: &mut egui::Ui,
		id: &str,
		rect: egui::Rect,
		region: &mut Region,
		edge: ResizeEdge,
		direction: f32,
	) {
		if !region.resizable {
			return;
		}
		let handle_size = 8.0;
		let handle = match edge {
			ResizeEdge::Left => egui::Rect::from_min_max(
				egui::pos2(rect.left() - handle_size / 2.0, rect.top()),
				egui::pos2(rect.left() + handle_size / 2.0, rect.bottom()),
			),
			ResizeEdge::Right => egui::Rect::from_min_max(
				egui::pos2(rect.right() - handle_size / 2.0, rect.top()),
				egui::pos2(rect.right() + handle_size / 2.0, rect.bottom()),
			),
			ResizeEdge::Top => egui::Rect::from_min_max(
				egui::pos2(rect.left(), rect.top() - handle_size / 2.0),
				egui::pos2(rect.right(), rect.top() + handle_size / 2.0),
			),
			ResizeEdge::Bottom => egui::Rect::from_min_max(
				egui::pos2(rect.left(), rect.bottom() - handle_size / 2.0),
				egui::pos2(rect.right(), rect.bottom() + handle_size / 2.0),
			),
		};
		Self::resize_handle(ui, id, handle, edge, |delta| {
			region.size = (region.size + delta * direction).clamp(region.min_size, region.max_size);
		});
	}
	pub fn cursor_target(&self, pos: Option<egui::Pos2>, layout: &VeLayout) -> CursorTarget {
		let Some(pos) = pos else {
			return CursorTarget::None;
		};
		if self.activity_bar.open && layout.activity_bar.contains(pos) {
			CursorTarget::ActivityBar
		} else if self.dock_left.open && layout.dock_left.contains(pos) {
			CursorTarget::DockLeft
		} else if layout.primary_bar.contains(pos) {
			CursorTarget::PrimaryBar
		} else if layout.secondary_bar.contains(pos) {
			CursorTarget::SecondaryBar
		} else if layout.main.contains(pos) {
			CursorTarget::Main
		} else if self.bottom_panel.open && layout.bottom_panel.contains(pos) {
			CursorTarget::BottomPanel
		} else if self.dock_right.open && layout.dock_right.contains(pos) {
			CursorTarget::DockRight
		} else if layout.status_bar.contains(pos) {
			CursorTarget::StatusBar
		} else {
			CursorTarget::None
		}
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
	#[default]
	MainEditor,
	SidePanel,
	CenterGrid,
	Unknown,
}
// pub struct TabbedSidebar<T> {
// 	pub active_tab: T,
// 	pub tabs: Vec<(T, String)>,
// }
// impl<T> TabbedSidebar<T>
// where
// 	T: Clone + PartialEq,
// {
// 	pub fn new(active_tab: T, tabs: Vec<(T, impl Into<String>)>) -> Self {
// 		Self {
// 			active_tab,
// 			tabs: tabs
// 				.into_iter()
// 				.map(|(tab, label)| (tab, label.into()))
// 				.collect(),
// 		}
// 	}
// 	pub fn draw<F>(&mut self, ui: &mut egui::Ui, mut draw_content: F)
// 	where
// 		F: FnMut(&mut egui::Ui, &T),
// 	{
// 		ui.horizontal(|ui| {
// 			for (tab, label) in &self.tabs {
// 				if ui
// 					.selectable_label(self.active_tab == *tab, label)
// 					.clicked()
// 				{
// 					self.active_tab = tab.clone();
// 				}
// 			}
// 		});
// 		ui.separator();
// 		egui::ScrollArea::vertical()
// 			.auto_shrink([false, false])
// 			.show(ui, |ui| {
// 				draw_content(ui, &self.active_tab);
// 			});
// 	}
// }
// impl<R, T> ViewTrait<R, E> for TabbedSidebar<T>
// where
// 	R: Runtime,
// 	T: Clone + PartialEq + 'static,
// {
// 	fn draw(&mut self, ui: &mut egui::Ui, _ctx: &mut AppContext<'_, R, E>) {
// 		self.draw(ui, |ui, _tab| {
// 			// content gets supplied by the owning view
// 		});
// 	}
// 	fn update(&mut self, ctx: &mut AppContext<'_, R, E>) {}
// 	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R, E>) {}
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct Sidebar<T> {
// 	pub active_tab: T,
// 	pub tabs: Vec<(T, String)>,
// }
// impl<T> Sidebar<T>
// where
// 	T: Clone + PartialEq,
// {
// 	pub fn new(active_tab: T, tabs: Vec<(T, impl Into<String>)>) -> Self {
// 		Self {
// 			active_tab,
// 			tabs: tabs
// 				.into_iter()
// 				.map(|(tab, label)| (tab, label.into()))
// 				.collect(),
// 		}
// 	}
// 	pub fn draw<F>(&mut self, ui: &mut egui::Ui, mut content: F)
// 	where
// 		F: FnMut(&mut egui::Ui, &T),
// 	{
// 		ui.horizontal(|ui| {
// 			for (tab, label) in &self.tabs {
// 				if ui
// 					.selectable_label(self.active_tab == *tab, label)
// 					.clicked()
// 				{
// 					self.active_tab = tab.clone();
// 				}
// 			}
// 		});
// 		ui.separator();
// 		egui::ScrollArea::vertical().show(ui, |ui| {
// 			content(ui, &self.active_tab);
// 		});
// 	}
// }
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Tab {
	#[default]
	Problem,
	Solutions,
	Submissions,
}
