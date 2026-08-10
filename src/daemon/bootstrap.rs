use crate::{daemon::config::derive_runtime_context, prelude::*};

fn manifest_path() -> Result<PathBuf, Error> {
	Ok(engine_data_dir()?.join("manifest.json"))
}

fn symbols_path() -> Result<PathBuf, Error> {
	Ok(engine_data_dir()?.join("symbols.json"))
}

fn write_manifest() -> Result<(), Error> {
	let path = manifest_path()?;

	let manifest = Manifest {
		installed_version: ESTATE_VERSION.to_string(),
		schema_version: SCHEMA_VERSION,
		initialized_at: std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs() as i64,
	};

	fs::write(path, serde_json::to_string_pretty(&manifest)?)?;

	Ok(())
}

fn write_symbols() -> Result<(), Error> {
	let path = symbols_path()?;
	let estate = resolver::global_estate_dir()?;

	let symbols = serde_json::json!({
			"symbols": [
					{
							"id": "estate.root",
							"path": estate.to_string_lossy(),
							"immutable": true,
							"doc": "Global user estate directory"
					},
					{
							"id": "estate.engine",
							"path": resolver::engine_data_dir()?.to_string_lossy(),
							"immutable": true,
							"doc": "Estate engine runtime directory"
					},
					{
							"id": "estate.manifest",
							"path": path.to_string_lossy(),
							"doc": "Estate engine installation state"
					}
			]
	});

	fs::write(path, serde_json::to_string_pretty(&symbols)?)?;

	Ok(())
}

fn create_user_estate() -> Result<(), Error> {
	let root = global_estate_dir()?;

	let files = [
		(
			root.join("README.md"),
			r#"# Estate

This directory is your global estate.

Use it for:
- bookmarks
- snippets
- templates
- knowledge
- reusable resources
"#,
		),
		(
			root.join("estate.md"),
			r#"# My Estate

This file describes my global workspace.
"#,
		),
	];

	fs::create_dir_all(&root)?;

	for (path, content) in files {
		if !path.exists() {
			fs::write(path, content)?;
		}
	}

	Ok(())
}

fn run_migrations(from: u32) -> Result<()> {
	println!("Running schema migrations from version {from}");

	match from {
		0 => {
			// migrate old layout
		}
		1 => {}
		_ => {}
	}

	Ok(())
}

fn run_version_migrations(from: &str) -> Result<()> {
	println!("Upgrading estate {from} → {ESTATE_VERSION}");

	Ok(())
}

#[derive(Serialize, Deserialize)]
struct Manifest {
	installed_version: String,
	schema_version: u32,
	initialized_at: i64,
}

fn ensure_initialized() -> Result<()> {
	let manifest_file = manifest_path()?;

	if !manifest_file.exists() {
		//
		// Engine initialization
		//
		initialize::init()?;

		//
		// User-facing estate initialization
		//
		create_user_estate()?;

		write_manifest()?;
		write_symbols()?;

		return Ok(());
	}

	let raw = fs::read_to_string(&manifest_file)?;
	let manifest: Manifest = serde_json::from_str(&raw)?;

	if manifest.schema_version < SCHEMA_VERSION {
		run_migrations(manifest.schema_version)?;
	}

	if manifest.installed_version != ESTATE_VERSION {
		run_version_migrations(&manifest.installed_version)?;
		write_manifest()?;
	}

	Ok(())
}

pub fn bootstrap(workspace: PathBuf) {
	let _ = workspace;

	if let Err(err) = ensure_initialized() {
		eprintln!("Estate bootstrap failed: {err}");
		return;
	}

	derive_runtime_context();
}
