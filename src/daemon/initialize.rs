use std::{fs, io::Result};

use crate::daemon::resolver::engine_data_dir;

pub fn init() -> Result<()> {
	let root = engine_data_dir()?;
	let dirs = [
		root.join("registry"),
		root.join("views"),
		root.join("cache/indexes"),
		root.join("cache/projections"),
		root.join("logs"),
	];

	for dir in dirs {
		fs::create_dir_all(dir)?;
	}

	let files = [
		(
			root.join("config.toml"),
			r#"# estate configuration

version = 1
"#,
		),
		(
			root.join("state.json"),
			r#"{
  "starts": 0,
  "started_at": 0,
  "longest_run": 0
}
"#,
		),
		(root.join("daemon.pid"), ""),
		(root.join("socket"), ""),
		(root.join("registry/graph.db"), ""),
		(
			root.join("views/default.toml"),
			r#"# Default view

name = "default"
"#,
		),
		(root.join("logs/estate.log"), ""),
	];

	for (path, contents) in files {
		if !path.exists() {
			fs::write(path, contents)?;
		}
	}

	println!("Initialized estate engine at {}", root.display());

	Ok(())
}
