#![allow(warnings)]

use std::{env, fs, io, path::Path};
// https://material-foundation.github.io/material-theme-builder/
use estate::prelude::*;

fn main() -> io::Result<()> {
	let template_root = Path::new(TEMPLATE_PATH);
	let destination = env::current_dir()?;
	materialize_static(template_root, &destination)?;
	materialize_rendered(template_root, &destination)?;
	// generate_artifacts(
	// 	// &estate,
	// 	template_root & destination,
	// )?;
	Ok(())
}
fn materialize_static(template_root: &Path, destination: &Path) -> io::Result<()> {
	let files = [
		("public/index.html", None),
		("public/docs/versions.json", None),
		("public/docs/v0.0.1", None),
		("public/docs/v0.0.2", None),
		("public/docs/v0.0.3", None),
	];

	for (source, target) in files {
		let target = target.unwrap_or(source);

		let source_path = template_root.join(source);
		let target_path = destination.join(target);

		println!("COPY");
		println!("  from: {}", source_path.display());
		println!("  to:   {}", target_path.display());

		if !source_path.exists() {
			return Err(io::Error::new(
				io::ErrorKind::NotFound,
				format!("Template path does not exist: {}", source_path.display()),
			));
		}

		if source_path.is_dir() {
			copy_dir(&source_path, &target_path)?;
		} else {
			if let Some(parent) = target_path.parent() {
				fs::create_dir_all(parent)?;
			}

			fs::copy(&source_path, &target_path)?;
		}
	}

	Ok(())
}
fn materialize_rendered(template_root: &Path, destination: &Path) -> io::Result<()> {
	let source = template_root.join("config.toml");
	let target = destination.join("config.toml");
	let template = fs::read_to_string(&source)?;
	let rendered = template
		.replace("{{ name }}", "my-project")
		.replace("{{ version }}", "0.1.0");
	if let Some(parent) = target.parent() {
		fs::create_dir_all(parent)?;
	}
	fs::write(target, rendered)?;
	Ok(())
}

pub struct Template {
	pub name: &'static str,
	pub root: &'static str,
}
impl Template {
	pub const WORKSPACE: Self = Self {
		name: "workspace",
		root: "templates/workspace",
	};

	pub const LOI: Self = Self {
		name: "loi",
		root: "templates/loi",
	};
}
pub fn materialize(
	template: &Template,
	template_root: &Path,
	destination: &Path,
) -> std::io::Result<()> {
	let source = template_root.join(template.root);
	copy_dir(&source, destination)
}
fn copy_dir(source: &Path, destination: &Path) -> std::io::Result<()> {
	fs::create_dir_all(destination)?;
	for entry in fs::read_dir(source)? {
		let entry = entry?;
		let source_path = entry.path();
		let destination_path = destination.join(entry.file_name());
		if source_path.is_dir() {
			copy_dir(&source_path, &destination_path)?;
		} else {
			fs::copy(&source_path, &destination_path)?;
		}
	}
	Ok(())
}
pub struct Materializer<'a> {
	pub template_root: &'a Path,
	pub destination: &'a Path,
}
impl<'a> Materializer<'a> {
	pub fn new(template_root: &'a Path, destination: &'a Path) -> Self {
		Self {
			template_root,
			destination,
		}
	}

	// ─────────────────────────────────────────────
	// Stage 1: Static
	// ─────────────────────────────────────────────

	pub fn static_file(&self, source: impl AsRef<Path>, target: impl AsRef<Path>) -> io::Result<()> {
		let source = self.template_root.join(source);
		let target = self.destination.join(target);
		if let Some(parent) = target.parent() {
			fs::create_dir_all(parent)?;
		}
		fs::copy(source, target)?;
		Ok(())
	}

	// ─────────────────────────────────────────────
	// Stage 2: Rendered
	// ─────────────────────────────────────────────

	pub fn rendered_file(
		&self,
		source: impl AsRef<Path>,
		target: impl AsRef<Path>,
		content: &str,
	) -> io::Result<()> {
		let target = self.destination.join(target);
		if let Some(parent) = target.parent() {
			fs::create_dir_all(parent)?;
		}
		fs::write(target, content)?;
		Ok(())
	}

	// ─────────────────────────────────────────────
	// Stage 3: Generated
	// ─────────────────────────────────────────────

	pub fn generated_file(
		&self,
		target: impl AsRef<Path>,
		content: impl AsRef<[u8]>,
	) -> io::Result<()> {
		let target = self.destination.join(target);
		if let Some(parent) = target.parent() {
			fs::create_dir_all(parent)?;
		}
		fs::write(target, content)?;
		Ok(())
	}
}
