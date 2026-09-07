use estate::prelude::*;

fn main_daemon() -> Result<()> {
	let host = Host::init()?;
	let mut app = App::new(host)?;
	let result = app.run();
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
