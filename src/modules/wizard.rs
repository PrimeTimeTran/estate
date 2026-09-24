// https://github.com/hjl1045/keyopen
// https://github.com/hjl1045/keyopen/blob/main/docs/superpowers/specs/2026-07-05-macos-shortcut-helper-design.md?utm_source=chatgpt.com

use ratatui::{Frame, layout::Rect};
use std::time::Duration;
use tray_icon::TrayIcon;

/// --- Enums
///
pub enum Action {
	// Navigation(NavigationAction),
	OpenView(ViewId),
	// OpenOverlay(OverlayId),
	CloseOverlay,
	// Execute(CommandId),
	ExecuteCommand(CommandId),
	Focus(SurfaceId),
	SelectTab(TabId),
	Quit,
}
pub enum AdornmentContent {
	Text(String),
	// Icon(Icon),
	Action(ActionItem),
	View(ViewId),
}
pub enum AdornmentPosition {
	Leading,
	Trailing,
}
pub enum BannerKind {
	Info,
	Success,
	Warning,
	Error,
	Prompt,
}
pub enum BindingOverride {
	Default,
	Custom(Vec<KeyBinding>),
	Disabled,
}
pub enum BindingState<T> {
	Default(T),
	Custom(T),
	Disabled,
}
pub enum Condition {
	True,
	False,

	Equals(ContextKey, ContextValue),
	Exists(ContextKey),

	Not(Box<Condition>),

	All(Vec<Condition>),
	Any(Vec<Condition>),
}
pub enum Focus {
	Main,
	Surface(SurfaceId),
	Overlay(OverlayId),
}
pub enum KeySequence {
	Single(KeyEvent),
	Chord(Vec<KeyEvent>),
	Double(KeyEvent),
	Hold(KeyCode),
}
struct KeyEvent;
struct KeyCode;
pub struct Tooltip;
pub enum NotificationKind {
	Info,
	Success,
	Warning,
	Error,
}
pub enum OverlayContent {
	Palette(Palette),
	Switcher(Switcher),
	Picker(Picker),
	Menu(Menu),
	Modal(Modal),
	Tooltip(Tooltip),
	Notification(Notification),
	Progress(ProgressOverlay),
}
pub enum OverlayContentInteractive {
	Palette(Palette),
	Switcher(Switcher),
	Picker(Picker),
	Menu(Menu),
	Modal(Modal),
	ShortcutHelp(ShortcutHelp),
}
pub enum OverlayContentPassive {
	Notification(Notification),
	Progress(ProgressOverlay),
}
pub enum OverlayFocus {
	None,
	Focusable,
	Focused,
}
pub enum OverlayLifetime {
	Manual,
	Duration(Duration),
	UntilDismissed,
}
pub enum OverlayPersistence {
	Transient,
	// automatically disappears
	Persistent,
	// remains until application/user explicitly removes it
	Pinned,
	// user has explicitly promoted a transient thing
	// into a persistent presentation
}
pub enum OverlayPlacement {
	Centered,
	Top,
	Bottom,
	Left,
	Right,
	TopLeft,
	TopRight,
	BottomLeft,
	BottomRight,

	Anchor(Rect),

	Custom,
}
pub enum OverlayModality {
	NonModal,
	Modal,
}
pub enum Presentation {
	Window(WindowId),
	Overlay(OverlayId),
	Surface(SurfaceId),
}
pub enum PresentationScope {
	Viewport,
	Window,
	Workspace,
	Desktop,
}
pub enum SidebarPosition {
	Left,
	Right,
}
pub enum TabId {
	Project,
	Environment,
	Advanced,
	Review,
}
pub enum WindowContent {
	Main,
	Surface(SurfaceId),
	Dock(DockId),
	Overlay(OverlayId),
	Custom(ViewId),
}
pub enum WizardAction {
	Back,
	Next,
	Cancel,
	Finish,
}
pub enum WizardStep {
	Project,
	Type,
	Features,
	Options,
	Review,
}

/// --- Traits: Contracts
///
pub trait Host {
	type Context;
	fn present(&self, presentation: Presentation);
	fn notify(&self, notification: Notification);
	fn open_window(&self, window: WindowId);
	fn close_window(&self, window: WindowId);
	fn tray(&self) -> Option<&TrayIcon>;
}
pub trait Interactive {
	fn handle_key(&mut self, key: KeyEvent) -> Action;
}
pub trait Tray {
	// Estate's semantic tray API
}
pub trait View {
	fn render(&self, frame: &mut Frame, area: Rect);
}

/// --- Impls: Business Logic
///
impl OverlayContent {
	pub fn render(&self, frame: &mut Frame, area: Rect) {
		match self {
			Self::Palette(p) => p.render(frame, area),
			Self::Switcher(s) => s.render(frame, area),
			Self::Picker(p) => p.render(frame, area),
			Self::Menu(m) => m.render(frame, area),
			Self::Modal(m) => m.render(frame, area),
			Self::Tooltip(t) => t.render(frame, area),
			Self::Notification(n) => n.render(frame, area),
			Self::Progress(p) => p.render(frame, area),
		}
	}
}
impl OverlayManager {
	pub fn active(&mut self) -> Option<&mut Overlay> {
		self.overlays.last_mut()
	}
	pub fn push(&mut self, overlay: Overlay) {
		self.overlays.push(overlay);
	}

	// pub fn dismiss(&mut self, id: OverlayId) {
	// 	self.overlays.retain(|overlay| overlay.id != id);
	// }
}
impl View for BottomBar {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for ProgressOverlay {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for Sidebar {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for TopBar {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for Menu {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for Modal {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for Notification {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for Palette {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for Picker {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for Switcher {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl View for Tooltip {
	fn render(&self, frame: &mut Frame, area: Rect) {
		// ...
	}
}
impl<C, S> Wizard<C, S> {
	fn handle_key(&mut self, key: KeyEvent) -> Result<WizardAction, anyhow::Error> {
		match self.step {
			// WizardStep::Project => self.project_key(key),
			// WizardStep::Type => self.type_key(key),
			// WizardStep::Features => self.features_key(key),
			// WizardStep::Options => self.options_key(key),
			// WizardStep::Review => self.review_key(key),
			_ => {
				todo!("handle_key")
			}
		}
	}
	fn next(&mut self) -> Result<(), anyhow::Error> {
		match self.step {
			WizardStep::Project => {
				// self.state.validate_project()?;
			}

			WizardStep::Type => {
				// self.state.validate_type()?;
			}

			WizardStep::Features => {
				// self.state.validate_features()?;
			}

			_ => {}
		}
		// self.step = self.step.next();
		Ok(())
	}
	fn render(&mut self, frame: &mut Frame) {
		match self.step {
			// WizardStep::Project => self.render_project(frame),
			// WizardStep::Type => self.render_type(frame),
			// WizardStep::Features => self.render_features(frame),
			// WizardStep::Options => self.render_options(frame),
			// WizardStep::Review => self.render_review(frame),
			_ => {
				todo!("render")
			}
		}
	}
	pub fn create_loop(&mut self) -> Result<(), anyhow::Error> {
		return Ok(());
		loop {
			// TODO: render with a real Ratatui Frame
			// self.render(frame);
			// TODO: read a real key event
			// let action = self.handle_key(key)?;

			break;
			// self.render(&mut terminal)?;
			// // read key...
			// let action = self.handle_key(key)?;
			// match action {
			// 	WizardAction::Back => self.back()?,
			// 	WizardAction::Next => self.next()?,
			// 	WizardAction::Cancel => break,
			// 	WizardAction::Finish => {
			// 		self.finish()?;
			// 		break;
			// 	}
			// }
		}

		Ok(())
	}
	pub fn finish(&mut self) -> Result<(), anyhow::Error> {
		todo!("finish")
	}
}
impl WizardStep {
	pub fn next(self) -> Self {
		match self {
			WizardStep::Project => WizardStep::Type,
			WizardStep::Type => WizardStep::Features,
			WizardStep::Features => WizardStep::Options,
			WizardStep::Options => WizardStep::Review,
			WizardStep::Review => WizardStep::Review,
		}
	}
}
/// --- Structs: Entities
///
pub struct App<C, S> {
	_context: C,
	_state: S,
	pub commands: CommandRegistry,
	pub navigation: Navigation,
	pub overlays: OverlayManager,
	pub shell: Shell,
	pub shortcuts: ShortcutRegistry,
	pub windows: WindowManager,
}
pub struct ActionItem {
	// pub action: Action,
	// pub label: String,
	// pub shortcut: Option<KeyBinding>,
	pub command: CommandId,
	pub label: Option<String>,
}
pub struct Adornment {
	pub position: AdornmentPosition,
	pub content: AdornmentContent,
}
pub struct AppShell<C, S> {
	context: C,
	state: S,
	layout: LayoutConfig,
	tabs: Vec<Tab>,
	active_tab: usize,
}
pub struct Banner {
	pub kind: BannerKind,
	pub message: String,
}
pub struct BottomBar {
	pub content: String,
}
pub struct Breadcrumbs {
	pub items: Vec<Breadcrumb>,
}
pub struct Breadcrumb;
pub struct BottomRegion {
	pub banner: Option<Banner>,
	pub status: Option<StatusBar>,
}
struct ContextKey;
struct ContextValue;
pub struct Chrome {
	pub leading: Vec<Adornment>,
	pub trailing: Vec<Adornment>,
}
pub struct Collection<T> {
	pub items: Vec<T>,
	pub state: CollectionState,
}
pub struct CollectionState {
	// pub visible: bool,
	// pub docked: bool,
	pub selected: usize,
	pub query: Option<String>,
}
pub struct Command {
	// = What can the system do?
	pub id: CommandId,
	pub title: String,
	pub action: Action,

	pub context: CommandContext,
	pub when: Condition,
}
pub struct CommandConfig {
	pub bindings: BindingOverride,
	pub enabled: bool,
}
pub struct CommandContext {
	// = Where does it make sense?
	pub scopes: Vec<ContextId>,
}
pub struct CommandId(String);
pub struct CommandRegistry;
pub struct ContextId;
pub struct Dock {
	// Dock is a container; Surface is the thing being presented.
	pub id: DockId,
	pub items: Collection<DockItem>,
	pub chrome: Chrome,
}
pub struct DockId(String);
pub struct DockItem {
	pub id: DockItemId,
	pub title: String,
	pub view: ViewId,
}
pub struct DockItemId;
pub struct ElementId;
pub struct FilterState {
	pub query: String,
}
pub struct Icon {}
pub struct KeyHint {
	pub key: KeyBinding,
	pub label: String,
}
pub struct KeyBinding {
	// = How can the user invoke it?
	pub sequence: KeySequence,
	pub command: CommandId,
}
pub struct Keymap {
	pub id: KeymapId,
	pub bindings: Vec<KeyBinding>,
}
pub struct KeymapId;
pub struct KeymapLayer {
	pub keymap: KeymapId,
	pub priority: i32,
	pub context: CommandContext,
}
pub struct LayoutConfig {
	pub top: RegionConfig,
	pub left: RegionConfig,
	pub bottom: RegionConfig,
}
pub struct List<T> {
	pub items: Vec<T>,
	pub selection: Selection,
	pub filter: Option<FilterState>,
}
pub struct Menu;
pub struct Modal;
pub struct Navigation {
	pub focus: Focus,
	pub history: NavigationHistory,
}
pub struct NavigationHistory {
	pub focus: History<Focus>,
	pub selection: History<SelectionRef>,
}
pub struct History<T> {
	_type: T,
}
pub struct Selection;
pub struct SelectionRef;
pub struct MainRegion {}
pub struct Notification {
	pub title: Option<String>,
	pub message: String,
	pub kind: NotificationKind,
	// pub duration: Option<Duration>,
	pub actions: Vec<ActionItem>,
}
pub struct Overlay {
	pub id: OverlayId,

	// What is being shown?
	pub content: OverlayContent,

	// Where does it appear?
	pub placement: OverlayPlacement,

	// How does it behave?
	pub behavior: OverlayBehavior,
	// But don't add OverlayLayout until you actually need it.
	// pub layout: OverlayLayout,
}
pub struct OverlayId;
pub struct OverlayBehavior {
	pub persistence: OverlayPersistence,
	pub lifetime: OverlayLifetime,
	pub focus: OverlayFocus,
	pub dismissal: OverlayDismissalPolicy,
	pub modality: OverlayModality,
}
// pub struct OverlayConfig {
// 	pub placement: OverlayPlacement,
// 	pub width: Constraint,
// 	pub height: Constraint,
// 	pub modal: bool,
// 	pub dismissible: bool,
// }
pub struct OverlayDismissalPolicy {
	pub escape: bool,
	pub click_outside: bool,
	pub click_inside: bool,
	pub focus_loss: bool,
	pub explicit: bool,
}
pub struct OverlayManager {
	// pub stack: Vec<Overlay>,
	pub overlays: Vec<Overlay>,
}
pub struct Palette {
	pub query: String,
	pub results: Vec<PaletteItem>,
	pub selected: usize,
}
pub struct PaletteItem {}
// pub struct PanelFrame {
// 	pub leading: Vec<Adornment>,
// 	pub content: Box<dyn View>,
// 	pub trailing: Vec<Adornment>,
// }
pub struct ProgressOverlay;
pub struct ProjectState {
	pub name: String,
	pub project_type: Option<ProjectType>,
	pub features: Vec<Feature>,
	pub initialize: bool,
	pub action: Option<Action>,
}
pub struct Feature;
pub struct ProjectType;
pub struct Picker {
	pub items: Vec<PickerItem>,
	pub selected: usize,
	pub query: String,
}
pub struct PickerItem {
	pub title: String,
	pub subtitle: Option<String>,
}
pub struct Rail {
	pub chrome: Chrome,
	pub items: Collection<RailItem>,
	pub state: RailState,
}
pub struct RailItem;
pub struct RailState {
	pub visible: bool,
	pub selected: usize,
	pub focused: bool,
}
pub struct RegionConfig {
	pub visible: bool,
	pub height: Option<u16>,
}
pub struct SelectionState {
	pub selected: usize,
}
// Where does persistent UI live?
pub struct Shell {
	pub top: TopRegion,
	pub left: SideRegion,
	pub main: MainRegion,
	pub right: SideRegion,
	pub bottom: BottomRegion,
}
pub struct ShortcutHelp;
pub struct SideRegion {
	// Rails render inside the viewport/window.
	pub rail: Rail,

	// Feature containers presented within this region.
	// Docks are containers that can be presented within this region.
	pub docks: Vec<Dock>,
}
// pub struct Selection<T> {
// 	pub items: Vec<T>,
// 	pub selected: usize,
// }
pub struct Sidebar {
	// A sidebar is a presentation surface. A rail is one possible
	// navigation mechanism for selecting surfaces within it. A
	// dock is one possible container for composing surfaces within it.
	pub tabs: Vec<Tab>,
	pub active: usize,
}
pub struct SidebarConfig {
	pub width: u16,
	pub position: SidebarPosition,
	pub show_numbers: bool,
	pub show_icons: bool,
	pub show_descriptions: bool,
}
pub struct ShortcutRegistry {
	pub bindings: Vec<KeyBinding>,
}
pub struct StatusBar {}
// Q: What persistent feature occupies 'this' space?
// A: A persistent presentation of a view.
pub struct Surface {
	// Dock is a container; Surface is the thing being presented.
	pub id: SurfaceId,
	pub chrome: Chrome,
	pub view: Box<dyn View>,
}
pub struct SurfaceId(String);
pub struct SurfaceState {
	pub visible: bool,
	pub focused: bool,
}
pub struct Switcher {
	pub items: Vec<SwitcherItem>,
	pub selected: usize,
}
pub struct SwitcherItem {
	pub id: ViewId,
	pub title: String,
	pub subtitle: Option<String>,
	pub icon: Option<Icon>,
}
// pub struct Tab<T> {
// 	pub id: TabId,
// 	pub title: String,
// 	pub content: T,
// }
pub struct Tab {
	pub id: TabId,
	pub title: String,
	pub view: ViewId,
}
pub struct TabBar {
	pub chrome: Chrome,

	pub tabs: Vec<Tab>,
	// pub active: usize,
}
pub struct TabState {
	pub active: usize,
}
pub struct TopRegion {
	pub primary: Option<TabBar>,
	pub secondary: Option<Breadcrumbs>,
}
pub struct TopBar {
	pub content: String,
}
pub struct Role;
pub struct Subrole;
pub struct Point;
pub struct Size;
struct UiElement {
	role: Role,
	subrole: Option<Subrole>,
	position: Point,
	size: Size,
	parent: Option<ElementId>,
	children: Vec<ElementId>,
	window: WindowId,
	actions: Vec<Action>,
}
pub struct ViewId(String);
pub struct Window {
	pub id: WindowId,
	pub shell: Shell,
	pub state: WindowState,
}
pub struct WindowId;
pub struct WindowState {
	pub visible: bool,
	pub focused: bool,
	pub maximized: bool,
	pub minimized: bool,
}
pub struct WindowManager {
	pub windows: Vec<Window>,
	pub active: WindowId,
}
pub struct Wizard<C, S> {
	pub context: C,
	pub state: S,
	pub step: WizardStep,
}
