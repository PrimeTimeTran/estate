pub struct Run {
	pub id: Uuid,
	pub dir: PathBuf,
}

impl Run {
	pub fn new(root: &Path) -> std::io::Result<Self> {
		let id = Uuid::new_v4();
		let dir = root.join(id.to_string());

		std::fs::create_dir_all(&dir)?;

		Ok(Self { id, dir })
	}
}
