// Logs
// https://www.youtube.com/watch?v=I2mWnh66Bkg
#![allow(warnings)]

use estate::prelude::*;

#[path = "./crates/demand.rs"]
mod demand_main;

// #[path = "./crates/sdlc.rs"]
// mod sdlc_main;

fn main() -> Result<()> {
	demand_main::main();
	// sdlc_main::main();
	Ok(())
}
