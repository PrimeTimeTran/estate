// use crate::{LAYOUT as config, e, prelude::*, theme::palette, ui::Layout};

use crate::{e, prelude::*, ui::Layout};

/// A screen-level coordinator.
///
/// A `Screen` defines the composition and behavior of a complete application
/// screen. It configures which Views are placed into which Layout regions and
/// coordinates state or behavior that spans multiple Views.
///
/// A Screen does not own the physical UI regions themselves. Those are owned
/// by the Layout it configures.
///
/// Screen-level state may be shared by multiple Views without being promoted
/// to global application state.
pub(crate) trait Screen<R: Runtime> {
	fn configure(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>);

	fn update(&mut self, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>);

	fn event(&mut self, event: &e::Event, layout: &mut Layout<R>, ctx: &mut AppContext<'_, R>);
}

/// The reusable spatial structure of an application UI.
///
/// A `Layout` defines the common regions that a screen may use, such as
/// activity bars, docks, the main area, bottom panels, and status bars.
///
/// A Layout does not know which Screen is using it and does not own
/// screen-specific behavior. A Screen configures the Layout by placing Views
/// into its regions.
///
/// Not every Screen needs to use every region.
pub(crate) trait LayoutTrait<R: Runtime> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>);

	fn update(&mut self, ctx: &mut AppContext<'_, R>);

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>);
}

/// A logical location within a Layout.
///
/// A `Region` represents a slot in the application's spatial structure,
/// such as the main area, a dock, an activity bar, or a bottom panel.
///
/// A Region determines where a Panel is presented, but does not own the
/// Panel's screen-specific content or panel-level state.
///
/// Regions belong to a Layout and may be unused by a particular Screen.
pub(crate) trait RegionTrait<R: Runtime> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>);

	fn update(&mut self, ctx: &mut AppContext<'_, R>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {}
}

/// A presentation surface that hosts one or more Views.
///
/// A `Panel` is an interactive UI container that can be placed within a
/// Layout Region. It owns panel-level concerns such as visibility, sizing,
/// focus, input handling, shortcuts, and presentation state.
///
/// A Panel may be docked into a Region or detached into a floating window
/// without changing the Views it contains.
///
/// Panel-level state belongs to the Panel rather than global application
/// state.
pub(crate) trait PanelTrait<R: Runtime> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>);

	fn update(&mut self, ctx: &mut AppContext<'_, R>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {}
}

/// A reusable, stateful piece of UI that can be placed into a Region or Panel.
///
/// A View owns the UI state specific to the content it presents and may
/// contain Components. Views can be moved between Layout regions without
/// changing what they represent.
///
/// Views may also share state with other Views when that state belongs to
/// their shared screen or feature scope rather than global application state.
pub(crate) trait ViewTrait<R: Runtime> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>);

	fn update(&mut self, ctx: &mut AppContext<'_, R>);

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>);
}

/// A smaller reusable UI unit composed inside a View.
///
/// A Component owns only the state and behavior needed by that piece of UI.
/// Components may be composed freely within Views and may themselves be
/// stateful, without requiring their state to live in global application
/// state.
pub(crate) trait ComponentTrait<R: Runtime> {
	fn draw(&mut self, ui: &mut egui::Ui, ctx: &mut AppContext<'_, R>);

	fn update(&mut self, ctx: &mut AppContext<'_, R>) {}

	fn event(&mut self, event: &e::Event, ctx: &mut AppContext<'_, R>) {}
}
