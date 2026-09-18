pub mod json;
#[path = "./toml.rs"]
pub mod toml_file;

pub use json::*;
pub use toml_file::*;
