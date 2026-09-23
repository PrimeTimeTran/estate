// Logs
// https://www.youtube.com/watch?v=I2mWnh66Bkg
#![allow(warnings)]

use estate::prelude::*;

#[path = "./crates/demand.rs"]
mod demand_main;

#[path = "[gestures].rs"]
mod gestures;

fn main() -> Result<()> {
	// demand_main::main();
	gestures::main();
	Ok(())
}
