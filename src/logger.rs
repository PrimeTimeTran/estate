use crate::{app::RuntimeMode, daemon::engine_data_dir};
use std::{
	fs::{self, OpenOptions}, io, path::{Path, PathBuf},
};
use tracing::{Instrument, debug, error, info, info_span, warn};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(tail: bool) -> anyhow::Result<()> {
	use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

	let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("estate=info"));

	let stdout = fmt::layer().with_target(true).with_thread_ids(true);

	tracing_subscriber::registry()
		.with(filter)
		.with(stdout)
		.init();

	Ok(())
}

pub fn init() -> anyhow::Result<()> {
	tracing_subscriber::registry()
		.with(EnvFilter::from_default_env())
		.with(tracing_subscriber::fmt::layer())
		.init();

	Ok(())
}
