use crate::{native::poc::McpClient, prelude::*};
use Error;
use std::os::unix::fs::MetadataExt;
#[derive(Debug, Default, Clone)]
pub struct FileInfo {
	pub content: String,
	pub extension: Option<String>,
	pub inode: String,
	pub is_directory: bool,
	pub language: Option<String>,
	pub modified_at: Option<SystemTime>,
	pub name: String,
	pub path: String,
	pub size: u64,
}
impl FileInfo {
	pub fn from_path(path: &Path) -> Result<Self> {
		let metadata = std::fs::symlink_metadata(path)?;

		let is_directory = metadata.is_dir();

		let content = if metadata.is_file() {
			std::fs::read_to_string(path).unwrap_or_default()
		} else {
			String::new()
		};

		let extension = path
			.extension()
			.and_then(|ext| ext.to_str())
			.map(String::from);

		let name = path
			.file_name()
			.and_then(|name| name.to_str())
			.unwrap_or_default()
			.to_string();

		let path = path.to_string_lossy().into_owned();

		let inode = metadata.ino().to_string();

		let language = extension.as_deref().and_then(language_from_extension);

		Ok(Self {
			content,
			extension,
			inode,
			is_directory,
			language,
			modified_at: metadata.modified().ok(),
			name,
			path,
			size: metadata.len(),
		})
	}
}
fn language_from_extension(extension: &str) -> Option<String> {
	let language = match extension {
		"rs" => "Rust",
		"js" => "JavaScript",
		"jsx" => "JavaScript",
		"ts" => "TypeScript",
		"tsx" => "TypeScript",
		"py" => "Python",
		"go" => "Go",
		"java" => "Java",
		"c" => "C",
		"h" => "C",
		"cpp" => "C++",
		"cc" => "C++",
		"cxx" => "C++",
		"hpp" => "C++",
		"cs" => "C#",
		"swift" => "Swift",
		"kt" => "Kotlin",
		"kts" => "Kotlin",
		"rb" => "Ruby",
		"php" => "PHP",
		"sh" => "Shell",
		"bash" => "Shell",
		"zsh" => "Shell",
		"fish" => "Shell",
		"html" => "HTML",
		"css" => "CSS",
		"scss" => "SCSS",
		"json" => "JSON",
		"toml" => "TOML",
		"yaml" => "YAML",
		"yml" => "YAML",
		"xml" => "XML",
		"md" => "Markdown",
		"sql" => "SQL",
		_ => return None,
	};

	Some(language.to_string())
}

#[derive(Debug, Default, Clone)]
pub struct AgentTools {
	pub fs: FileSystemTool,
	pub mcp: McpClient,
}

#[derive(Debug, Default, Clone)]
pub struct Vfs;

impl Vfs {
	pub fn new() -> Self {
		Self
	}

	pub fn search_files(&self, query: &str) -> Result<Vec<FileInfo>, Error> {
		dbg!("Search VFS for files matching query {}", query);

		Ok(vec![FileInfo::default()])
	}

	pub fn create_file(&self, path: &str, content: &str) -> Result<(), Error> {
		dbg!("Create file in VFS");

		println!("Created file: {} ({} bytes)", path, content.len());

		Ok(())
	}

	pub fn read_file(&self, path: &str) -> Result<String, Error> {
		dbg!("Read file contents from VFS");

		Ok(format!(
			"// Placeholder content for {}\n\nfn main() {{}}\n",
			path
		))
	}

	pub fn write_file(&self, path: &str, content: &str) -> Result<(), Error> {
		dbg!("write_file");
		if let Some(parent) = std::path::Path::new(path).parent() {
			std::fs::create_dir_all(parent)?;
		}
		std::fs::write(path, content)?;
		Ok(())
	}
}

#[derive(Default, Debug, Clone)]
pub struct FileSystemTool {
	pub vfs: Vfs,
}

impl FileSystemTool {
	pub fn search(&self, query: &str) -> Result<Vec<FileInfo>> {
		self.vfs.search_files(query)
	}

	pub fn read(&self, path: &str) -> Result<String> {
		self.vfs.read_file(path)
	}

	pub fn write(&self, path: &str, content: &str) -> Result<()> {
		self.vfs.write_file(path, content)
	}

	pub fn create(&self, path: &str, content: &str) -> Result<()> {
		self.vfs.create_file(path, content)
	}
}
