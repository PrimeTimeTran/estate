use estate::prelude::{ logger, * };

fn main() -> Result<()> {
	let cli = cli::context::parse();
	let mut config = LogConfig::load()?;
	config.apply_cli(&cli)?;
	logger::init_logging(&config)?;
	let trace = Tracer::new("app");
	let mut flow = trace.flow("init");
	flow.info("App::new");
	let mut app = NativeApp::new()?;
	flow.info(">>> BEFORE app.run()");
	let result = app.run(cli);
	flow.info(">>> AFTER app.run(): {result}");
	flow.info(">>> main returning");
	std::process::exit(match result {
		Ok(()) => 0,
		Err(error) => {
			eprintln!("{error:#}");
			1
		}
	});
}
