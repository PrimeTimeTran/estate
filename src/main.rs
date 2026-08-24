// use estate::prelude::{logger, *};

// fn main() -> anyhow::Result<()> {
// 	let cli = cli::context::parse();

// 	let mut config = LogConfig::load()?;
// 	config.apply_cli(&cli)?;
// 	logger::init_logging(&config)?;

// 	let mut app = App::new()?;

// 	app.run(cli)
// }
