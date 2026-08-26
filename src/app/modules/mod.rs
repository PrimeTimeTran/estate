pub(crate) mod runtime;
pub(crate) mod monitor;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod monitor_native;
#[cfg(not(target_arch = "wasm32"))]
pub use monitor_native::{ * };
