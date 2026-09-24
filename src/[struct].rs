//! ## [Struct]
//!
//! All structs in a file looks
//! messy but helps us quickly spot competing abstractions early.
//!
//! What's the different between state, app state, native state, web state for example?
//!
//! The difference may not be apparent to either reader or author... So having structs default
//! next to each other helps us to see where they align, diverge, and can be squashed or merged.
//!
//! When the identifier has stabilized and we're confident this
//! is a meaningful difference then it's appropriate to move the definition to a domain specific dir.
//!
use crate::prelude::*;

/// ## [AppState]
///
#[derive(Debug, Default, Clone)]
pub struct AppState {
	pub problem: ProblemState,
	pub problems: ProblemListState,
}

/// ## [C]
///
/// Type state architecture placeholder for [Ctx] or [Context][traits::Context]
///
/// Generic type state of structs with C by default enables
/// more easily hiding and enabling of methods/params at compile time.
///
/// When the abstract context has become concrete silo it's capabilities
/// for little cost to make code safer via documentation & preventing the app to even compile.
///
pub struct C;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct EstateState {
	pub revision: u64,
	pub starts: u64,
	pub longest_run: u64,
	pub status_checks: u64,
	pub started_at: u64,
	pub events_processed: u64,
	pub tasks_completed: u64,
	pub tasks_created: u64,
	pub files_indexed: u64,
	pub session: Session,
	pub jobs: VecDeque<Job>,
}

#[cfg(feature = "web")]
#[derive(Clone)]
pub struct HostClock;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
#[derive(Clone)]
pub struct HostClock {
	#[cfg(feature = "native")]
	pub handle: tokio::runtime::Handle,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Inode;
pub struct Linux;
pub struct MacOS;

#[derive(Debug, Default, Clone)]
pub struct ProblemListState {
	pub error: Option<String>,
	pub items: Vec<StoredProblem>,
	pub loading: bool,
}

#[derive(Debug, Default, Clone)]
pub struct ProblemState {
	pub error: Option<String>,
	pub loading: bool,
	pub value: Option<StoredProblem>,
}
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
/// ## [S]
///
/// Typestate placeholder for state.
///
/// State vs Context is like "Nature vs Nurture",there is no perfect answer to what drives what.
/// Every state depends on some context which depending on how you think of it, might be considered "state" as well.
///
/// So for now, in order to implement a Type State system robustly, we're going to agree that all apps/processes must come from a context.
///
/// Linux, MacOS, Windows, they're all contexts in which the app can run so we begin our app with that assumption for modeling more robustly.
///
#[derive(Debug)]
pub struct S<C> {
	pub context: PhantomData<fn() -> C>,
	pub state: PhantomData<fn() -> C>,
	pub view: ViewType,
}

#[derive(Debug, Copy, Clone)]
pub struct State;

pub struct Windows;

pub static NEXT_RECEIVER_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct BroadcastReceiver<T> {
	pub id: u64,
	pub owner: &'static str,
	pub rx: broadcast::Receiver<T>,
}

impl ReceivesEvents for BroadcastReceiver<e::Event> {
	fn try_recv(&mut self) -> Option<e::Event> {
		match self.rx.try_recv() {
			Ok(event) => {
				tracing::debug!(?event.kind, "ScreenInstance::event");
				Some(event)
			}

			Err(TryRecvError::Empty) => {
				tracing::debug!(id = self.id, "RECEIVER EMPTY");
				None
			}

			Err(err) => {
				tracing::error!(id = self.id, ?err, "RECEIVER ERROR");
				None
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct BroadcastSender<T> {
	pub tx: broadcast::Sender<T>,
}

impl BroadcastSender<e::Event> {
	pub fn new(tx: broadcast::Sender<e::Event>) -> Self {
		Self { tx }
	}
}

impl SendsEvents for BroadcastSender<e::Event> {
	fn send(&self, event: e::Event) {
		let _ = self.tx.send(event);
	}
}
