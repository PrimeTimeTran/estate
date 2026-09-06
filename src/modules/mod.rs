/// [Modules]
/// This should encapsulate logic that isn't "app" specific but 
/// also encapsulates larger systems that are bigger than a single
/// target. Event is a leading contender right now.
/// 

// Whats a mod? Something bigger than a mod, model, behavior.
// Consider this dir for "event". It's both client, server, native.
//
// Decided to move runtime out of this nesting because every target needs immediate access so nesting didnt
// feel right.
//
// pub(crate) mod runtime;
// pub use runtime::*;
