use estate::prelude::*;

fn main() -> Result<()> {
	let host = Host::init()?;
	let _app = App::new(host);
	Ok(())
}
