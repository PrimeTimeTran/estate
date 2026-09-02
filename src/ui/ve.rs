use crate::{LAYOUT as config, e, prelude::*, theme::palette};
// pub trait Veable<R: Runtime> {
// 	// A type-erased container for any concrete `Veable`.
// 	//
// 	// `Box<dyn Veable>` stores the concrete implementation on the heap while
// 	// exposing only the `Veable` interface to callers. This allows different
// 	// concrete implementations to be substituted without changing the code
// 	// which consumes them.
// 	//
// 	// Top left to bottom right ordering for mental model.
// 	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>);
// 	fn update(&mut self, _ctx: &mut AppContext<'_, R>) {}
// 	fn event(&mut self, _event: &e::Event, _ctx: &mut AppContext<'_, R>) {}
// }
// pub struct Ve<R: Runtime> {
// 	pub activity_bar: Panel<R>,
// 	pub dock_left: Panel<R>,
// 	pub main: Panel<R>,
// 	pub primary_bar: Panel<R>,
// 	pub secondary_bar: Panel<R>,
// 	pub bottom_panel: Panel<R>,
// 	pub status_bar: Panel<R>,
// 	pub dock_right: Panel<R>,
// }
pub struct Layout<R: Runtime> {
    pub activity_bar: Panel<R>,
    pub dock_left: Panel<R>,
    pub main: Panel<R>,
    pub primary_bar: Panel<R>,
    pub secondary_bar: Panel<R>,
    pub bottom_panel: Panel<R>,
    pub status_bar: Panel<R>,
    pub dock_right: Panel<R>,
}
impl<R: Runtime> LayoutTrait<R> for Layout<R> {
  fn draw(
		&mut self,
		ui: &mut egui::Ui,
		ctx: &mut AppContext<'_, R>,
	) {}

	fn update(
		&mut self,
		ctx: &mut AppContext<'_, R>,
	) {}

	fn event(
		&mut self,
		event: &e::Event,
		ctx: &mut AppContext<'_, R>,
	) {}
}
impl<R: Runtime> Layout<R> {
  fn draw(
    &mut self,
		// screen: &mut dyn Screen<R>,
		ui: &mut egui::Ui,
		ctx: &mut AppContext<'_, R>,
  ) {
  }
}
impl<R: Runtime> Layout<R> {
	// Rust uses ownership,borrowing, and lifetimes to determine when values
	// may be safely destroyed, allowing memory to be reclaimed deterministically
	// without a garbage collector.
	pub fn new() -> Self {
	  let view = ProblemScreen::new();
		Self {
		  main: Panel::from_config(view, Region::content(), &PanelState::new(true, 0.0)),
			activity_bar: Panel::from_config(
				ActivityBar::new(),
				Region::fixed(config.activity_bar.effective_size()),
				&config.activity_bar,
			),
			dock_left: Panel::from_config(
        TabbedSidebar::new(
            Tab::Problem,
            vec![
                (Tab::Problem, "Problem"),
                (Tab::Solutions, "Solutions"),
                (Tab::Submissions, "Submissions"),
            ],
        ),
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
				DebugPanel::new("DebugPanel"),
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
	// pub fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
	// 	// Forwards the drawing contract to the concrete implementation.
	// 	//
	// 	// `Ve` doesn't know how the view is drawn. It only knows that the
	// 	// contained implementation satisfies `Veable`.
	// 	let mouse_pos = ui.input(|i| i.pointer.hover_pos());
	// 	let available = ui.available_rect_before_wrap();
	// 	ui.painter().rect_filled(available, 0.0, LAYOUT.bg);
	// 	let layout = self.calculate_region_boundaries(available);
	// 	let VeLayout {
	// 		activity_bar,
	// 		bottom_panel,
	// 		dock_left,
	// 		dock_right,
	// 		main,
	// 		primary_bar,
	// 		secondary_bar,
	// 		status_bar,
	// 	} = layout;
	// 	let cursor_target = self.cursor_target(mouse_pos, &layout);
	// 	ctx.input = IOState {
	// 		cursor_pos: mouse_pos,
	// 		cursor_target,
	// 		shift_held: ui.input(|i| i.modifiers.shift),
	// 		ctrl_held: ui.input(|i| i.modifiers.ctrl),
	// 		alt_held: ui.input(|i| i.modifiers.alt),
	// 		command_held: ui.input(|i| i.modifiers.command),
	// 		primary_down: ui.input(|i| i.pointer.primary_down()),
	// 	};
	// 	if self.dock_left.open {
	// 		Self::draw_panel(ui, ctx, dock_left, &mut self.dock_left);
	// 		Self::resize_region(
	// 			ui,
	// 			"dock_left_resize",
	// 			dock_left,
	// 			&mut self.dock_left.region,
	// 			ResizeEdge::Right,
	// 			1.0,
	// 		);
	// 	}
	// 	Self::draw_panel(ui, ctx, primary_bar, &mut self.primary_bar);
	// 	Self::draw_panel(ui, ctx, secondary_bar, &mut self.secondary_bar);
	// 	Self::draw_panel(ui, ctx, main, &mut self.main);
	// 	if self.bottom_panel.open {
	// 		Self::draw_panel(ui, ctx, bottom_panel, &mut self.bottom_panel);
	// 		Self::resize_region(
	// 			ui,
	// 			"bottom_panel_resize",
	// 			bottom_panel,
	// 			&mut self.bottom_panel.region,
	// 			ResizeEdge::Top,
	// 			-1.0,
	// 		);
	// 	}
	// 	if self.dock_right.open {
	// 		Self::draw_panel(ui, ctx, dock_right, &mut self.dock_right);
	// 		Self::resize_region(
	// 			ui,
	// 			"dock_right_resize",
	// 			dock_right,
	// 			&mut self.dock_right.region,
	// 			ResizeEdge::Left,
	// 			-1.0,
	// 		);
	// 	}
	// 	Self::draw_panel(ui, ctx, status_bar, &mut self.status_bar);
	// }
	fn draw_view(
		ui: &mut egui::Ui,
		rect: egui::Rect,
		view: &mut dyn ViewTrait<R>,
		ctx: &mut AppContext<'_, R>,
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
		ctx: &mut AppContext<'_, R>,
		rect: egui::Rect,
		panel: &mut Panel<R>,
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
		// let config = LAYOUT;
		// =========================================================
		// Bottom Status Bar
		// =========================================================
		let status_bar_height = config.status_bar.effective_size();
		let activity_bar_width = if self.activity_bar.open {
			self.activity_bar.region.size
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
		let tabs_height = config.primary_bar.effective_size();
		let breadcrumbs_height = config.secondary_bar.effective_size();
		let min_main_height = 100.0;
		// =========================================================
		// Tabs
		// =========================================================
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
	fn resize_handle(ui: &mut egui::Ui, id: &str, rect: egui::Rect, mut resize: impl FnMut(f32)) {
		let cursor = if id == "bottom_panel_resize" {
			egui::CursorIcon::ResizeVertical
		} else {
			egui::CursorIcon::ResizeHorizontal
		};
		let id = egui::Id::new(id);
		let response = ui.interact(rect, id, egui::Sense::drag());
		if response.hovered() || response.dragged() {
			ui.ctx().set_cursor_icon(cursor);
		}
		let hovered = response.hovered();
		let dragged = response.dragged();
		let stroke = if hovered || dragged {
			ui.visuals().widgets.active.bg_stroke
		} else {
			ui.visuals().widgets.noninteractive.bg_stroke
		};
		ui.painter().rect_filled(rect, 0.0, stroke.color);
		if dragged {
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
		let handle = match edge {
			ResizeEdge::Left => egui::Rect::from_min_max(
				egui::pos2(rect.left() - 3.0, rect.top()),
				egui::pos2(rect.left() + 3.0, rect.bottom()),
			),
			ResizeEdge::Right => egui::Rect::from_min_max(
				egui::pos2(rect.right() - 3.0, rect.top()),
				egui::pos2(rect.right() + 3.0, rect.bottom()),
			),
			ResizeEdge::Top => egui::Rect::from_min_max(
				egui::pos2(rect.left(), rect.top() - 3.0),
				egui::pos2(rect.right(), rect.top() + 3.0),
			),
			ResizeEdge::Bottom => egui::Rect::from_min_max(
				egui::pos2(rect.left(), rect.bottom() - 3.0),
				egui::pos2(rect.right(), rect.bottom() + 3.0),
			),
		};
		Self::resize_handle(ui, id, handle, |delta| {
			let delta = match edge {
				ResizeEdge::Left | ResizeEdge::Right => delta,
				ResizeEdge::Top | ResizeEdge::Bottom => delta,
			};
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
// impl<R: Runtime> Veable<R> for Ve<R> {
// 	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
// 		self.main.draw(ui, ctx);
// 		// self.activity_bar.draw(ui, ctx);
// 		// self.dock_left.draw(ui, ctx);
// 		// self.primary_bar.draw(ui, ctx);
// 		// self.secondary_bar.draw(ui, ctx);
// 		// self.bottom_panel.draw(ui, ctx);
// 		// self.status_bar.draw(ui, ctx);
// 		// self.dock_right.draw(ui, ctx);
// 	}
// }
// pub struct Ve<R: Runtime> {
// 	///      A type-erased container for any concrete `Veable`.
// 	///
// 	///      `Box<dyn Veable>` stores the concrete implementation on the heap while
// 	///      exposing only the `Veable` interface to callers. This allows different
// 	///      concrete implementations to be substituted without changing the code
// 	///      which consumes them.
// 	// Top left to bottom right ordering for mental model.
// 	// Top left to bottom right ordering for mental model.
// 	// pub activity_bar: Region<R>,
// 	// pub dock_left: Panel<R>,
// 	pub main: Region<R>,
// 	// pub primary_bar: Region<R>,
// 	// pub secondary_bar: Region<R>,
// 	// pub bottom_panel: Panel<R>,
// 	// pub status_bar: Region<R>,
// 	// pub dock_right: Panel<R>,
// }
// impl<R: Runtime> Ve<R> {
// 	pub fn new(view: impl Veable<R> + 'static) -> Self {
// 		Self {
// 			main: Region::content(view),
// 		}
// 	}
// }
// impl<R: Runtime> Veable<R> for Region<R> {
// 	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
// 		// Region itself controls layout/presentation.
// 		//
// 		// For now, simply delegate to the contained view.
// 		self.content.draw(ui, ctx);
// 	}
// 	fn update(&mut self, ctx: &mut AppContext<'_, R>) {
// 		self.content.update(ctx);
// 	}
// 	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {
// 		self.content.event(event, ctx);
// 	}
// }
///      A named, interactive view that occupies a region.
///
///      Panels add interaction and lifecycle behavior to a Region.
///      They may be opened, closed, overlaid, auto-hidden, moved,
///      or potentially detached from their parent layout.
pub struct DebugPanel {
	pub title: String,
}
impl DebugPanel {
	pub fn new(title: impl Into<String>) -> Self {
		Self {
			title: title.into(),
		}
	}
}
impl<R: Runtime> ViewTrait<R> for DebugPanel {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>) {
		ui.vertical_centered(|ui| {
			ui.heading(&self.title);
			ui.separator();
			ui.label(format!(
				"{} × {}",
				ui.available_width(),
				ui.available_height()
			));
		});
	}
	fn update(
		&mut self,
		ctx: &mut AppContext<'_, R>,
	){}

	fn event(
		&mut self,
		event: &e::Event,
		ctx: &mut AppContext<'_, R>,
	){}
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
	#[default]
	MainEditor,
	SidePanel,
	CenterGrid,
	Unknown,
}
pub struct TabbedSidebar<T> {
    pub active_tab: T,
    pub tabs: Vec<(T, String)>,
}
impl<T> TabbedSidebar<T>
where
    T: Clone + PartialEq,
{
    pub fn new(active_tab: T, tabs: Vec<(T, impl Into<String>)>) -> Self {
        Self {
            active_tab,
            tabs: tabs
                .into_iter()
                .map(|(tab, label)| (tab, label.into()))
                .collect(),
        }
    }
    pub fn draw<F>(&mut self, ui: &mut egui::Ui, mut draw_content: F)
    where
        F: FnMut(&mut egui::Ui, &T),
    {
        ui.horizontal(|ui| {
            for (tab, label) in &self.tabs {
                if ui
                    .selectable_label(self.active_tab == *tab, label)
                    .clicked()
                {
                    self.active_tab = tab.clone();
                }
            }
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                draw_content(ui, &self.active_tab);
            });
    }
}
impl<R, T> ViewTrait<R> for TabbedSidebar<T>
where
    R: Runtime,
    T: Clone + PartialEq + 'static,
{
    fn draw(&mut self, ui: &mut egui::Ui, _ctx: &mut AppContext<'_, R>) {
        self.draw(ui, |ui, _tab| {
            // content gets supplied by the owning view
        });
    }
    fn update(
		&mut self,
		ctx: &mut AppContext<'_, R>,
	){}

	fn event(
		&mut self,
		event: &e::Event,
		ctx: &mut AppContext<'_, R>,
	){}

}
#[derive(Debug, Clone, PartialEq)]
pub struct Sidebar<T> {
    pub active_tab: T,
    pub tabs: Vec<(T, String)>,
}
impl<T> Sidebar<T>
where
    T: Clone + PartialEq,
{
    pub fn new(active_tab: T, tabs: Vec<(T, impl Into<String>)>) -> Self {
        Self {
            active_tab,
            tabs: tabs
                .into_iter()
                .map(|(tab, label)| (tab, label.into()))
                .collect(),
        }
    }
    pub fn draw<F>(&mut self, ui: &mut egui::Ui, mut content: F)
    where
        F: FnMut(&mut egui::Ui, &T),
    {
        ui.horizontal(|ui| {
            for (tab, label) in &self.tabs {
                if ui
                    .selectable_label(self.active_tab == *tab, label)
                    .clicked()
                {
                    self.active_tab = tab.clone();
                }
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            content(ui, &self.active_tab);
        });
    }
}
#[derive(Debug, Default, Clone, PartialEq)]
pub enum Tab {
    #[default]
    Problem,
    Solutions,
    Submissions,
}
