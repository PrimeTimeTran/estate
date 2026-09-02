use estate::prelude::*;

fn main() -> Result<()> {
	App::new(cli::context::parse())?.run()
}
