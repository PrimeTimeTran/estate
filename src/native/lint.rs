use crate::{data, prelude::*};

use revelation::analyzer::Workspace;

impl<R: Runtime> model::EstateEngine<R> {
	pub async fn format(self, args: &FormatArgs) -> Result<String, Error> {
		daemon::LintDaemon.run(&args).await;
		Ok("Success".to_string())
	}
}
