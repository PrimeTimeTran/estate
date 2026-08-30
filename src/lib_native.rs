// Fix 1 problem, introduce another
// Moving here breaks the ability for this to automatically find "./native/mod.rs"
// Was trying to make logic in lib.rs easier but that makes this worse
#[path = "./native/mod.rs"]
pub mod native;
#[path = "./native/mod.rs"]
pub use native::*;

#[path = "./event/mod.rs"]
pub mod event;
#[path = "./event/mod.rs"]
pub use event::*;

#[path = "./event/handler.rs"]
pub mod handler;
pub use handler::*;

#[path = "data/native.rs"]
pub mod native_data;
