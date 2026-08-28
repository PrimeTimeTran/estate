use crate::traits::one;

fn main() {
	one::context();
}

#[path = "sections/mod.rs"]
pub mod traits;
