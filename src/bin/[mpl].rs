use std::{
	path::{Path, PathBuf},
	process::Command,
	env,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// ## Steps to Global Install
///
/// 1. Build with install the bin
///
/// 	cargo install --path . --bin mpl
///
/// 	Maybe this: Because rust includes root crate even if the bin is it's own crate
///
/// 	cargo install --path . --bin mpl --features native
///
/// 	[Install/Overwrite current bin]
/// 	cargo install --path . --bin mpl --features native --force
///
/// 2. Confirm
///
/// 	which mpl
///
/// 	-> ~/.cargo/bin/mpl

fn main() -> Result<()> {
  let mut failed = vec![];
	let mut args = env::args().skip(1);
	let input_name = args.next().ok_or("usage: mpl <name> <url>")?;
	let url = args.next().ok_or("usage: mpl <name> <url>")?;
	let filename = normalize_filename(&input_name);
	let output = PathBuf::from(format!("/Users/future/consultants/[tmp]/{filename}.mp4"));
	println!("filename: {filename}");
	println!("url: {url}");
	println!("output: {}", output.display());
	// Try each strategy until one succeeds.
	if try1(&url, &output)? {
		println!("Download succeeded with strategy 1");
	} else if try2(&url, &output)? {
		println!("Download succeeded with strategy 2");
	} else if try3(&url, &output)? {
		println!("Download succeeded with strategy 3");
	} else {
	  failed.push(url);
		return Err("all download strategies failed".into());
	}
	post_process(&output)?;
	Ok(())
}

/// Normalize a human-readable name into a filename.
///
/// "Hello world"       -> "hello.world"
/// "My Cool File"      -> "my.cool.file"
/// "  Multiple   Words " -> "multiple.words"
fn normalize_filename(input: &str) -> String {
	input
		.split_whitespace()
		.map(|word| word.to_lowercase())
		.collect::<Vec<_>>()
		.join(".")
}

/// First download strategy.
///
/// Returns true only if the command exits successfully.
/// First download strategy.
fn try2(url: &str, output: &Path) -> Result<bool> {
	println!("Trying download strategy 1...");
	let status = Command::new("yt-dlp")
		.args([
			"--add-header",
			"sec-fetch-mode: cors",
			"--add-header",
			"sec-fetch-site: cross-site",
			"--add-header",
			"accept: */*",
			"--add-header",
			"Referer: ",
			"--add-header",
			"user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36",
		])
		.arg("-o")
		.arg(output)
		.arg(url)
		.status()?;

	Ok(status.success())
}

/// Second download strategy.
fn try1(url: &str, output: &Path) -> Result<bool> {
	println!("Trying download strategy 2...");

	let status = Command::new("yt-dlp")
		.arg("-o")
		.arg(output)
		.arg(url)
		.status()?;

	Ok(status.success())
}

/// Third download strategy.
fn try3(url: &str, output: &Path) -> Result<bool> {
	println!("Trying download strategy 3...");

	// Placeholder: this assumes the URL itself can be consumed by ffmpeg.
	let status = Command::new("ffmpeg")
		.arg("-i")
		.arg(url)
		.args(["-bsf:a", "aac_adtstoasc"])
		.args(["-vcodec", "copy"])
		.args(["-c", "copy"])
		.args(["-crf", "50"])
		.arg(output)
		.status()?;

	Ok(status.success())
}

fn post_process(path: &Path) -> Result<()> {
	fn validate_file(_path: &Path) -> Result<()> {
		todo!("validate_file");
	}
	fn repair_container(_path: &Path) -> Result<()> {
		todo!("repair_container");
	}
	fn validate_media(_path: &Path) -> Result<()> {
		todo!("validate_media");
	}
	fn refresh_quicklook(_path: &Path) -> Result<()> {
		todo!("refresh_quicklook");
	}

	println!("Post-processing: {}", path.display());

	// Step 1: Validate the file exists and isn't empty.
	validate_file(path)?;

	// Step 2: Repair / normalize the media container.
	repair_container(path)?;

	// Step 3: Validate that the resulting file is readable.
	validate_media(path)?;

	// Step 4: Ask macOS Quick Look to generate / refresh its preview.
	refresh_quicklook(path)?;

	Ok(())
}
