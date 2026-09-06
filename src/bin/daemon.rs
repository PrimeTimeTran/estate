use estate::prelude::*;

fn main_daemon() -> Result<()> {
	let cli = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;
	let trace = Tracer::new("app");
	let flow = trace.flow("init");
	flow.info("App::new");
	let app = App::new();
	flow.info(">>> Before app.run(): {app}");
	let result = app.run(cli);
	flow.info(">>> AFTER app.run(): {result}");
	std::process::exit(match result {
		Ok(()) => 0,
		Err(error) => {
			eprintln!("{error:#}");
			1
		}
	});
}

fn main() {
	let _result = main_daemon();
}
