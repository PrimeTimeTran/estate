#![allow(warnings)]

#[path = "./mod.rs"]
mod lifetime;

use lifetime::*;

fn main() {
	one::abstraction_of_references_and_pointers();
}
