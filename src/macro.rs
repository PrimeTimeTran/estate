//! [Macros]
//!
//! The macros used throughout our crate which are as generic as can be made
//! 


/// [problem_source!]
/// 
/// Parse  string file path quickly and easily.
#[macro_export]
macro_rules! problem_source {
	($problem:literal, Rust) => {
		include_str!(concat!("../data/problems/", $problem, "/success.rs"))
	};
	($problem:literal, Python) => {
		include_str!(concat!("../data/problems/", $problem, "/success.py"))
	};
	($problem:literal, JavaScript) => {
		include_str!(concat!("../data/problems/", $problem, "/success.js"))
	};
}

/// [section!]
///
/// Print a section header for grouping output when reading through dense text.
#[macro_export]
macro_rules! section {
	($title:expr) => {
		$crate::helpers::print_section($title, file!(), line!())
	};
}

pub use section;
