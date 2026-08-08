// #![allow(warnings)] // "dont allow warnings"

// Allows dead code(removes lint warnings)
pub mod _core;
#[allow(dead_code)]
#[allow(unused_imports)] // Silences unused imports

// #![allow(unused_must_use)]
// #![allow(unused_variables)]

// Add warnings
// #![warn(dead_code)]
// #![warn(unused_mut)]
// #![warn(unused_parens)]
// #![warn(unused_braces)]
// #![warn(unused_must_use)]
// #![warn(unused_assignments)]
// #![warn(unused_imports)]
// #![warn(unused_variables)]
pub mod daemon;
pub mod estate;
pub mod registry;
pub mod vfs;

pub use cli::context::*;
pub use revelation::*;
pub use vfs::*;
