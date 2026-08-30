pub mod time;

#[cfg(feature = "native")]
pub mod logger;

#[cfg(feature = "native")]
pub mod cargo;

#[macro_export]
macro_rules! doc {
	($text:expr) => {{ $text }};
}
