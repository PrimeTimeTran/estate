// Estate Engine CLI.

// Install:
// ```sh
// cargo install --path . --bin estate
// ```

// Usage:
// Start Estate Engine:
// ```sh
// estate start
// ```

// ```sh
// estate fmt path/to/file.rs
// ```

// lib.rs must have th efollowing for logger import here to use.
// pub mod tool;
// pub use tool::*;
use estate::prelude::*;

fn main() -> Result<()> {
	App::new(cli::context::parse())?.run()
}
