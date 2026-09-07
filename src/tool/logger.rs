use crate::{native::resolver::engine_data_dir, prelude::*};
/// Sorting order of VSCode
///
/// Enum > Macros > Functions > Impl > Structs
///
/// Makes organization easier cross IDE.
///
use std::fs::OpenOptions;
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::{
	EnvFilter, Layer, filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

/// ## [LogLevel]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
}

#[macro_export]
macro_rules! flow_warn {
	($flow:expr, $($arg:tt)*) => {
		$flow.warn(format!($($arg)*))
	};
}

pub fn init() -> Result<()> {
	tracing_subscriber::registry()
		.with(EnvFilter::from_default_env())
		.with(tracing_subscriber::fmt::layer())
		.init();
	Ok(())
}
pub fn init_logging(config: &LogConfig) -> Result<()> {
	let terminal_filter = config.terminal_filter()?;
	let terminal = fmt::layer()
		.without_time()
		.with_target(false)
		.with_thread_ids(false)
		.with_ansi(true)
		// .with_timer(fmt::time::SystemTime)
		.with_filter(terminal_filter);
	// let terminal = fmt::layer()
	// 	.with_target(true)
	// 	.with_thread_ids(false)
	// 	.with_ansi(true)
	// 	.with_filter(terminal_filter);
	// let terminal = fmt::layer()
	// 	.with_target(true)
	// 	.with_thread_ids(false)
	let file = if config.file.enabled {
		let path = engine_data_dir()?.join("estate.log");
		let writer = OpenOptions::new().create(true).append(true).open(path)?;
		Some(
			fmt::layer()
				.with_writer(writer)
				.with_target(true)
				.with_thread_ids(true)
				.with_thread_names(true)
				.with_file(true)
				.with_line_number(true)
				.with_ansi(false)
				.with_filter(LevelFilter::TRACE),
		)
	} else {
		None
	};
	// let subscriber = tracing_subscriber::registry().with(terminal);
	// if let Some(file) = file {
	// 	subscriber.with(file).init();
	// } else {
	// 	subscriber.init();
	// }
	tracing_subscriber::registry()
		.with(terminal)
		.with(file)
		.init();
	Ok(())
}
pub fn setup_logging() -> anyhow::Result<()> {
	let cli = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;
	// tracing::trace!("[dryrun] trace");
	// tracing::debug!("[dryrun] debug");
	// tracing::info!("[dryrun] info");
	// tracing::warn!("[dryrun] warn");
	// tracing::error!("[dryrun] error");
	Ok(())
}

static FLOW_ID: AtomicU64 = AtomicU64::new(0);

impl LogConfig {
	#[cfg(feature = "native")]
	pub fn apply_cli(&mut self, cli: &cli::context::Cli) -> Result<()> {
		match &cli.command {
			Some(cli::context::Command::Start {
				tail,
				foreground: false,
			}) => {
				if *tail {
					self.terminal.enabled = true;
				}
			}
			_ => {}
		}
		Ok(())
	}
	pub fn load() -> Result<Self> {
		let mut config = Self::default();
		if let Some(global) = Self::load_global()? {
			config.merge(global);
		}
		Ok(config)
	}
	fn terminal_filter(&self) -> Result<EnvFilter> {
		// let mut filter = EnvFilter::new("off");
		// for (target, level) in &self.targets {
		// 	let directive = format!("{target}={level}");
		// 	filter = filter.add_directive(directive.parse()?);
		// }
		// Ok(filter)
		// Enables targeting one or more namespaces
		let mut filter = EnvFilter::new(self.level.to_string());
		for (target, level) in &self.targets {
			let directive = format!("{target}={level}")
				.parse()
				.map_err(|e| anyhow::anyhow!("invalid log target `{target}`: {e}"))?;
			filter = filter.add_directive(directive);
		}
		Ok(filter)
	}

	fn load_from_cargo() -> Result<Option<LogConfig>> {
		let path = Path::new(env!("CARGO_MANIFEST_DIR"))
			.ancestors()
			.find_map(|dir| {
				let path = dir.join("estate.toml");
				path.exists().then_some(path)
			});
		let Some(path) = path else {
			return Ok(None);
		};
		let raw = fs::read_to_string(path)?;
		let manifest: CargoManifest = toml::from_str(&raw)?;
		Ok(manifest.logging)
	}
	fn workspace_cargo_toml() -> PathBuf {
		PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../estate.toml")
	}
	fn load_global() -> Result<Option<Self>> {
		let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.parent()
			.and_then(Path::parent)
			.ok_or_else(|| anyhow::anyhow!("could not find workspace root"))?
			.join("Estate.toml");
		let raw = fs::read_to_string(path)?;
		let cargo = toml::from_str::<CargoConfig>(&raw)?;
		Ok(Some(cargo.logging))
	}
	fn merge(&mut self, other: Self) {
		self.level = other.level;
		if other.terminal.enabled {
			self.terminal.enabled = true;
		}
		if other.terminal.level.is_some() {
			self.terminal.level = other.terminal.level;
		}
		if other.file.enabled {
			self.file.enabled = true;
		}
		if other.file.level.is_some() {
			self.file.level = other.file.level;
		}
		self.targets.extend(other.targets);
	}
}
impl Default for LogLevel {
	fn default() -> Self {
		Self::Info
	}
}
impl LogLevel {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Trace => "trace",
			Self::Debug => "debug",
			Self::Info => "info",
			Self::Warn => "warn",
			Self::Error => "error",
		}
	}
}
impl std::fmt::Display for LogLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self.as_str())
	}
}
impl Tracer {
	pub fn new(namespace: impl Into<String>) -> Self {
		Self {
			namespace: namespace.into(),
		}
	}
	pub fn flow(&self, name: impl Into<String>) -> TraceFlow {
		TraceFlow {
			namespace: self.namespace.clone(),
			name: name.into(),
		}
	}
	pub fn next_flow_id() -> u64 {
		FLOW_ID.fetch_add(1, Ordering::Relaxed) + 1
	}
}
impl TraceFlow {
	fn event(&self, level: LogLevel, message: impl std::fmt::Display) {
		let id = Tracer::next_flow_id();
		let prefix = format!("{}#{}:{}", self.name, id, self.namespace);

		match level {
			LogLevel::Trace => trace!("{prefix}: {message}"),
			LogLevel::Debug => debug!("{prefix}: {message}"),
			LogLevel::Info => info!("{prefix}: {message}"),
			LogLevel::Warn => warn!("{prefix}: {message}"),
			LogLevel::Error => error!("{prefix}: {message}"),
		}
	}

	pub fn trace(&self, message: impl std::fmt::Display) {
		self.event(LogLevel::Trace, message);
	}

	pub fn debug(&self, message: impl std::fmt::Display) {
		self.event(LogLevel::Debug, message);
	}
	/// [info]
	///
	/// "Always on" level
	///
	pub fn info(&self, message: impl std::fmt::Display) {
		self.event(LogLevel::Info, message);
	}

	pub fn warn(&self, message: impl std::fmt::Display) {
		self.event(LogLevel::Warn, message);
	}

	pub fn error(&self, message: impl std::fmt::Display) {
		self.event(LogLevel::Error, message);
	}
}

/// ## [CargoConfig]
#[derive(Debug, Deserialize)]
struct CargoConfig {
	#[serde(default)]
	logging: LogConfig,
}
/// ## [CargoManifest]
///
/// Minimal representation of a estate.toml manifest used by Estate.
/// This intentionally models only the fields Estate needs rather than
///
/// depending on Cargo's complete manifest schema.
#[derive(Debug, Deserialize)]
struct CargoManifest {
	#[serde(default)]
	logging: Option<LogConfig>,
}
/// ## [LogConfig]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct LogConfig {
	pub file: OutputConfig,
	pub level: LogLevel,
	pub targets: HashMap<String, LogLevel>,
	pub terminal: OutputConfig,
	// pub fields: LogFields,
	// pub window: OutputConfig,
}
/// ## [LogFieldConfig]
///
/// Fields which are configured by the CLI tracer
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LogFieldConfig {
	pub enabled: bool,
}
/// ## [LogFields]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LogFields {
	pub file: bool,
	pub line: bool,
	pub module: bool,
	pub target: bool,
	pub thread_id: bool,
	pub timestamp: bool,
}
/// ## [OutputConfig]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct OutputConfig {
	pub enabled: bool,
	pub level: Option<LogLevel>,
}
impl Default for OutputConfig {
	fn default() -> Self {
		Self {
			enabled: true,
			level: None,
		}
	}
}
/// ## [LogOptions]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LogOptions {
	pub file: Option<OutputOptions>,
	pub level: Option<LogLevel>,
	pub targets: Option<HashMap<String, LogLevel>>,
	pub terminal: Option<OutputOptions>,
}
/// ## [OutputOptions]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OutputOptions {}
/// ## [Tracer]
///
/// The object used to trace logs and flow through the app's lifecycle.
///
///
#[derive(Clone)]
pub struct Tracer {
	namespace: String,
}
/// ## [TraceFlow]
#[derive(Debug)]
pub struct TraceFlow {
	namespace: String,
	name: String,
}
