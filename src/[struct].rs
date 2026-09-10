//! ## [Struct]
//!
//!
//!
use crate::prelude::*;

/// ## [C]
///
/// Type state placeholder for context.
///
pub struct C;

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
	pub context: PhantomData<C>,
	pub state: PhantomData<C>,
	pub view: ViewType,
}

pub struct Renderer<C, S> {
	pub phantom: PhantomData<C>,
	pub state: S,
	pub view: ViewType,
	#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
	pub windows: Vec<AppWindow>,
	pub cancel: CancellationToken,
}

/// # Shared application state
///
/// ============================================================
#[derive(Debug, Default, Clone)]
pub struct AppState {
	pub problems: ProblemListState,
	pub problem: ProblemState,
}

#[derive(Debug, Default, Clone)]
pub struct ProblemListState {
	pub items: Vec<StoredProblem>,
	pub loading: bool,
	pub error: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ProblemState {
	pub value: Option<StoredProblem>,
	pub loading: bool,
	pub error: Option<String>,
}

pub struct Linux;
pub struct MacOS;
pub struct Windows;
