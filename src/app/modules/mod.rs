// Whats a mod? Something bigger than a mod, model, behavior.
// Consider this dir for "event". It's both client, server, native.
//
// Decided to move runtime out of this nesting because every target needs immediate access so nesting didnt
// feel right.
//
pub(crate) mod runtime;
pub use runtime::*;
