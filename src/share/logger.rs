use crate::prelude::*;
use crate::share::*;

use tracing::{ debug, error, info, info_span, trace, warn };
use tracing_subscriber::{
	EnvFilter,
	Layer,
	filter::LevelFilter,
	fmt,
	layer::SubscriberExt,
	util::SubscriberInitExt,
};

pub fn init_logging(config: &LogConfig) -> anyhow::Result<()> {
	let terminal_filter = config.terminal_filter()?;
	let terminal = fmt
		::layer()
		.without_time()
		.with_target(true)
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
	// let file = if config.file.enabled {
	// 	let path = engine_data_dir()?.join("estate.log");
	// 	let writer = OpenOptions::new().create(true).append(true).open(path)?;
	// 	Some(
	// 		fmt::layer()
	// 			.with_writer(writer)
	// 			.with_target(true)
	// 			.with_thread_ids(true)
	// 			.with_thread_names(true)
	// 			.with_file(true)
	// 			.with_line_number(true)
	// 			.with_ansi(false)
	// 			.with_filter(LevelFilter::TRACE),
	// 	)
	// } else {
	// 	None
	// };
	// let subscriber = tracing_subscriber::registry().with(terminal);
	// if let Some(file) = file {
	// 	subscriber.with(file).init();
	// } else {
	// 	subscriber.init();
	// }
	tracing_subscriber
		::registry()
		.with(terminal)
		// .with(file)
		.init();
	Ok(())
}
pub fn init() -> anyhow::Result<()> {
	tracing_subscriber
		::registry()
		.with(EnvFilter::from_default_env())
		.with(tracing_subscriber::fmt::layer())
		.init();
	Ok(())
}

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
impl LogConfig {
	#[cfg(feature = "native")]
	pub fn apply_cli(&mut self, cli: &cli::context::Cli) -> anyhow::Result<()> {
		match &cli.command {
			Some(cli::context::Command::Start { tail }) => {
				if *tail {
					self.terminal.enabled = true;
				}
			}
			_ => {}
		}
		Ok(())
	}
	fn terminal_filter(&self) -> anyhow::Result<EnvFilter> {
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
	pub fn load() -> anyhow::Result<Self> {
		let mut config = Self::default();
		if let Some(global) = Self::load_global()? {
			config.merge(global);
		}
		Ok(config)
	}
	fn load_from_cargo() -> anyhow::Result<Option<LogConfig>> {
		let path = Path::new(env!("CARGO_MANIFEST_DIR"))
			.ancestors()
			.find_map(|dir| {
				let path = dir.join("Estate.toml");
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
		PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Estate.toml")
	}
	fn load_global() -> anyhow::Result<Option<Self>> {
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
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

static FLOW_ID: AtomicU64 = AtomicU64::new(0);
#[derive(Clone)]
pub struct Tracer {
	namespace: String,
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
pub struct TraceFlow {
	namespace: String,
	name: String,
}
impl TraceFlow {
	fn event(&self, level: LogLevel, message: &str) {
		let id = Tracer::next_flow_id();
		let span = info_span!(
		"flow",
		flow = %format_args!("{}#{}.{}", self.namespace, id, self.name),
		);
		let _enter = span.enter();
		match level {
			LogLevel::Trace => trace!("{}", message),
			LogLevel::Debug => debug!("{}", message),
			LogLevel::Info => info!("{}", message),
			LogLevel::Warn => warn!("{}", message),
			LogLevel::Error => error!("{}", message),
		}
	}
	pub fn trace(&mut self, message: &str) {
		self.event(LogLevel::Trace, message);
	}
	pub fn debug(&mut self, message: &str) {
		self.event(LogLevel::Debug, message);
	}
	pub fn info(&mut self, message: &str) {
		self.event(LogLevel::Info, message);
	}
	pub fn warn(&mut self, message: &str) {
		self.event(LogLevel::Warn, message);
	}
	pub fn error(&mut self, message: &str) {
		self.event(LogLevel::Error, message);
	}
}

#[derive(Debug, Deserialize)]
struct CargoConfig {
	#[serde(default)]
	logging: LogConfig,
}
/// Minimal representation of a Estate.toml manifest used by Estate.
/// This intentionally models only the fields Estate needs rather than
///
/// depending on Cargo's complete manifest schema.
#[derive(Debug, Deserialize)]
struct CargoManifest {
	#[serde(default)]
	logging: Option<LogConfig>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OutputOptions {}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LogFieldConfig {
	pub enabled: bool,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LogFields {
	pub file: bool,
	pub line: bool,
	pub module: bool,
	pub target: bool,
	pub thread_id: bool,
	pub timestamp: bool,
}
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
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LogOptions {
	pub file: Option<OutputOptions>,
	pub level: Option<LogLevel>,
	pub targets: Option<HashMap<String, LogLevel>>,
	pub terminal: Option<OutputOptions>,
}
