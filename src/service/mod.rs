/// ## Service
///
/// These need to be available outside of Server because client/wasm code
/// also depends on the availability of the page helpers as well.
///
use crate::prelude::*;

pub mod event;
pub use event::*;

pub mod api;
pub use api::*;
