//! A virtual filesystem abstraction over Estate's resources.
//! The VFS provides a unified way to resolve, access, and navigate files and
//! directories represented within an Estate, independent of their underlying
//! storage or physical location.
//!
//! # VFS
//!
//! A virtual filesystem abstraction for navigating and accessing Estate
//! resources.
//!
//! The VFS presents Estate's resources through a filesystem-like interface,
//! decoupling consumers from the underlying storage, filesystem, or resource
//! representation.
//!
//! It provides the foundation for resolving paths and nodes, accessing
//! resources, and maintaining filesystem state as the Estate changes.
use crate::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct Namespace {
	paths: HashMap<PathBuf, Uuid>,
}
impl Namespace {
	pub fn new() -> Self {
		Self {
			paths: HashMap::new(),
		}
	}
	pub fn insert(&mut self, path: PathBuf, inode: Uuid) {
		self.paths.insert(path, inode);
	}
	pub fn resolve(&self, path: &Path) -> Option<Uuid> {
		self.paths.get(path).copied()
	}
}

pub trait Vfs {
	// Lookup
	fn resolve_inode(&self, inode: Uuid) -> Result<Node>;

	// Mutation
	fn create(&mut self, parent: Uuid, name: &str, kind: NodeKind) -> Result<Node>;

	fn update(&mut self, node: Uuid, data: &[u8]) -> Result<()>;
	fn delete(&mut self, node: Uuid) -> Result<()>;

	// Synchronization
	fn upsert(&mut self, node: Node) -> Result<()>;

	// Cache
	fn invalidate(&mut self, node: Uuid);
}
#[derive(Debug, Clone)]
pub struct VirtualFileSystem {
	pub inodes: InodeStore,
	pub namespace: Namespace,
	pub root: Uuid,
}

impl VirtualFileSystem {
	pub fn new() -> Self {
		let mut inodes = InodeStore::new();

		let root = inodes.create(InodeKind::Directory);

		let mut namespace = Namespace::new();
		namespace.insert(PathBuf::from("/"), root);

		Self {
			inodes,
			namespace,
			root,
		}
	}
}

// impl Vfs for VirtualFileSystem {
// 	fn resolve_inode(&self, id: Uuid) -> Option<&Inode> {
// 		self.inodes.get(id)
// 	}
// 	fn invalidate(&mut self, id: Uuid) {
// 		self.inodes.remove(id);
// 	}
// }
// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct Node;
// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct NodeId;

// 	pub fn from_root(path: PathBuf) -> Self {
// 		let mut vfs = Self::new();
// 		let root = vfs.init_root();
// 		vfs.paths.insert(path, root);
// 		vfs
// 	}
// 	pub fn resolve_path(&self, path: &Path) -> Option<u64> {
// 		self.paths.get(path).copied()
// 	}
// 	pub fn resolve_uri(&self, uri: &Url) -> Option<u64> {
// 		let path = uri.to_file_path().ok()?;
// 		self.paths.get(&path).copied()
// 	}
// 	pub fn root_for_path(&self, path: &Path) -> Option<u64> {
// 		self
// 			.paths
// 			.iter()
// 			.filter(|(root, _)| path.starts_with(root))
// 			.max_by_key(|(root, _)| root.components().count())
// 			.map(|(_, id)| *id)
// 	}
// 	pub fn upsert_file(&mut self, uri: &Url, data: Vec<u8>) -> u64 {
// 		let path = uri.to_file_path().unwrap();
// 		if let Some(id) = self.paths.get(&path).copied() {
// 			self.write(id, data);
// 			return id;
// 		}
// 		let root = self.root_for_path(&path).expect("no virtual root mounted");
// 		let name = path.file_name().unwrap().to_string_lossy();
// 		let id = self.create_file(root, &name, data);
// 		self.paths.insert(path, id);
// 		id
// 	}
// 	pub fn init_root(&mut self) -> u64 {
// 		let id = self.store.next_id;
// 		self.store.next_id += 1;
// 		let inode = Inode {
// 			version: 0,
// 			meta: Meta {
// 				created_at: 0,
// 				modified_at: 0,
// 				size: 0,
// 			},
// 			name: "root".to_string(),
// 			kind: InodeKind::Directory(vec![]),
// 		};
// 		self.store.inodes.insert(id, inode);
// 		id
// 	}
// 	pub fn create_file(&mut self, parent: u64, name: &str, data: Vec<u8>) -> u64 {
// 		let id = self.import_file(name, data);
// 		if let Some(inode) = self.store.inodes.get_mut(&id) {
// 			inode.version = 1;
// 		}
// 		self.add_child(parent, id);
// 		id
// 	}
// 	pub fn import_file(&mut self, name: &str, content: Vec<u8>) -> u64 {
// 		let id = self.store.next_id;
// 		self.store.next_id += 1;
// 		let inode = Inode::new_file(name, content);
// 		self.store.inodes.insert(id, inode);
// 		id
// 	}
// 	pub fn create_dir(&mut self, _parent: u64, name: &str) -> u64 {
// 		self.import_dir(name)
// 	}
// 	pub fn import_dir(&mut self, name: &str) -> u64 {
// 		let id = self.store.next_id;
// 		self.store.next_id += 1;
// 		let inode = Inode::new_dir(name);
// 		self.store.inodes.insert(id, inode);
// 		id
// 	}
// 	pub fn read_url(&self, url: &Url) -> Result<Vec<u8>, String> {
// 		let id = self
// 			.lookup(url)
// 			.ok_or_else(|| format!("file not found: {url}"))?;
// 		self.try_read(id)
// 	}
// 	pub fn load_file(&mut self, path: PathBuf, content: Vec<u8>) -> u64 {
// 		let id = self.store.next_id;
// 		self.store.next_id += 1;
// 		let name = path.file_name().unwrap().to_string_lossy().to_string();
// 		let inode = Inode::new_file(&name, content);
// 		self.store.inodes.insert(id, inode);
// 		self.paths.insert(path, id);
// 		id
// 	}
// }
// impl Vfs {
// 	pub fn open(&mut self, uri: Url, text: String) -> u64 {
// 		let path = uri.to_file_path().expect("LSP URI must be a file URI");
// 		self.load_file(path, text.into_bytes())
// 	}
// 	pub fn try_open(&mut self, uri: Url, text: String) -> Result<u64, String> {
// 		let path = uri
// 			.to_file_path()
// 			.map_err(|_| "LSP URI must be a file URI".to_string())?;
// 		Ok(self.load_file(path, text.into_bytes()))
// 	}
// 	pub fn get_inode(&self, id: u64) -> Option<&Inode> {
// 		self.store.inodes.get(&id)
// 	}
// 	pub fn len(&self) -> usize {
// 		self.store.inodes.len()
// 	}
// 	pub fn read(&self, id: u64) -> Vec<u8> {
// 		self
// 			.store
// 			.inodes
// 			.get(&id)
// 			.map(|inode| match &inode.kind {
// 				InodeKind::File(data) => data.clone(),
// 				_ => vec![],
// 			})
// 			.unwrap_or_default()
// 	}
// 	pub fn try_read(&self, id: u64) -> Result<Vec<u8>, String> {
// 		self
// 			.store
// 			.inodes
// 			.get(&id)
// 			.ok_or_else(|| format!("inode {} not found", id))
// 			.and_then(|inode| match &inode.kind {
// 				InodeKind::File(data) => Ok(data.clone()),
// 				_ => Err(format!("inode {} is not a file", id)),
// 			})
// 	}
// 	pub fn lookup(&self, uri: &Url) -> Option<u64> {
// 		let path = uri.to_file_path().ok()?;
// 		self.paths.get(&path).copied()
// 	}
// 	pub fn write(&mut self, id: u64, data: Vec<u8>) {
// 		if let Some(inode) = self.store.inodes.get_mut(&id) {
// 			inode.version += 1;
// 			if let InodeKind::File(ref mut content) = inode.kind {
// 				*content = data;
// 			}
// 		}
// 		let mut affected_parents = Vec::new();
// 		for (parent, node) in &self.runtime.tree {
// 			if node.children.iter().any(|c| c.id == id) {
// 				affected_parents.push(*parent);
// 			}
// 		}
// 		self.runtime.tree.remove(&id);
// 		self.runtime.compiled.remove(&id);
// 		self.runtime.dirty.insert(id);
// 		for p in affected_parents {
// 			self.runtime.dirty.insert(p);
// 		}
// 		let graph = self.build_dependency_graph();
// 		if let Some(dependents) = graph.dependents.get(&id) {
// 			for &dep_id in dependents {
// 				self.runtime.compiled.remove(&dep_id);
// 				self.runtime.dirty.insert(dep_id);
// 			}
// 		}
// 	}
// }
// impl Vfs {
// 	pub fn extract_imports_from_source(&self, data: &[u8]) -> Vec<String> {
// 		let content = String::from_utf8_lossy(data);
// 		content
// 			.lines()
// 			.filter(|line| line.contains("import"))
// 			.filter_map(|line| {
// 				let start = line.find(['\'', '\"'])?;
// 				let end = line[start + 1..].find(['\'', '\"'])?;
// 				Some(line[start + 1..start + 1 + end].to_string())
// 			})
// 			.collect()
// 	}
// 	pub fn build_dependency_graph(&self) -> DependencyGraph {
// 		let mut dependents = HashMap::new();
// 		let mut dependencies = HashMap::new();
// 		for (&id, inode) in &self.store.inodes {
// 			if let InodeKind::File(data) = &inode.kind {
// 				for import in self.extract_imports_from_source(data) {
// 					if let Some(target_id) = self.resolve(id, &import) {
// 						dependencies
// 							.entry(id)
// 							.or_insert_with(HashSet::new)
// 							.insert(target_id);
// 						dependents
// 							.entry(target_id)
// 							.or_insert_with(HashSet::new)
// 							.insert(id);
// 					}
// 				}
// 			}
// 		}
// 		DependencyGraph {
// 			dependents,
// 			dependencies,
// 		}
// 	}
// 	pub fn resolve(&self, base_id: u64, path: &str) -> Option<u64> {
// 		let parent_id = self.find_parent_of(base_id)?;
// 		let target_name = path.trim_start_matches("./");
// 		self.list_children(parent_id).into_iter().find(|&id| {
// 			self
// 				.get_inode(id)
// 				.is_some_and(|node| node.name == target_name)
// 		})
// 	}
// 	pub fn find_parent_of(&self, child_id: u64) -> Option<u64> {
// 		self.store.inodes.iter().find_map(|(parent_id, inode)| {
// 			if let InodeKind::Directory(children) = &inode.kind
// 				&& children.contains(&child_id)
// 			{
// 				return Some(*parent_id);
// 			}
// 			None
// 		})
// 	}
// 	pub fn list_children(&self, id: u64) -> Vec<u64> {
// 		match self.store.inodes.get(&id) {
// 			Some(Inode {
// 				kind: InodeKind::Directory(children),
// 				..
// 			}) => children.clone(),
// 			_ => vec![],
// 		}
// 	}
// 	pub fn add_child(&mut self, parent: u64, child: u64) {
// 		if let Some(Inode {
// 			kind: InodeKind::Directory(children),
// 			..
// 		}) = self.store.inodes.get_mut(&parent)
// 		{
// 			children.push(child);
// 		}
// 		self.runtime.tree.remove(&parent);
// 	}
// 	pub fn analyze_source(&self, source: &[u8]) -> VfsResult<SourceAnalysis> {
// 		dbg!("VFS analyze_source, source!");
// 		let content = std::str::from_utf8(source)
// 			.map_err(|_| VfsError::SyntaxError("Invalid UTF-8".to_string()))?;
// 		let imports = self.extract_imports(content);
// 		let exports = self.extract_exports(content);
// 		Ok(SourceAnalysis { imports, exports })
// 	}
// 	pub fn extract_imports(&self, _content: &str) -> Vec<String> {
// 		vec![]
// 	}
// 	pub fn extract_exports(&self, _content: &str) -> Vec<String> {
// 		vec![]
// 	}
// }
// impl Vfs {
// 	pub fn compile(&mut self, id: u64) -> VfsResult<()> {
// 		let raw_source = self.read(id);
// 		if raw_source.starts_with(b"!!!") {
// 			self.runtime.compiled.remove(&id);
// 			return Err(VfsError::SyntaxError("Invalid source".to_string()));
// 		}
// 		let inode = self
// 			.store
// 			.inodes
// 			.get(&id)
// 			.ok_or(VfsError::InodeNotFound(id))?;
// 		let dep_sum = self.get_dependency_version_sum(id);
// 		let effective_version = inode.version + dep_sum;
// 		if let Some(cached) = self.runtime.compiled.get(&id)
// 			&& cached.source_version == effective_version
// 		{
// 			return Ok(());
// 		}
// 		let raw_source = self.read(id);
// 		dbg!("VFS compile about to call analyze_source!");
// 		let analysis = self.analyze_source(&raw_source)?;
// 		let compiled = CompiledNode::new(
// 			id,
// 			effective_version,
// 			raw_source,
// 			analysis.imports,
// 			analysis.exports,
// 		);
// 		self.runtime.compiled.insert(id, compiled);
// 		Ok(())
// 	}
// 	pub fn get_dependency_version_sum(&self, id: u64) -> u64 {
// 		let graph = self.build_dependency_graph();
// 		let mut sum = 0;
// 		if let Some(deps) = graph.dependencies.get(&id) {
// 			for &dep_id in deps {
// 				if let Some(inode) = self.store.inodes.get(&dep_id) {
// 					sum += inode.version;
// 				}
// 			}
// 		}
// 		sum
// 	}
// 	pub fn build_tree(&mut self, root: u64) -> Arc<TreeNode> {
// 		if let Some(cached) = self.runtime.tree.get(&root)
// 			&& !self.runtime.dirty.contains(&root)
// 		{
// 			return Arc::clone(cached);
// 		}
// 		let inode = self.store.inodes.get(&root).expect("missing inode");
// 		let version = inode.version;
// 		let name = inode.name.clone();
// 		let children_ids = match &inode.kind {
// 			InodeKind::Directory(c) => c.clone(),
// 			InodeKind::File(_) => vec![],
// 		};
// 		let mut children_nodes = Vec::new();
// 		let mut child_sigs = Vec::new();
// 		for child in children_ids {
// 			let child_tree = self.build_tree(child);
// 			child_sigs.push(child_tree.signature);
// 			children_nodes.push((*child_tree).clone());
// 		}
// 		let signature = self.compute_signature(root, version, &child_sigs);
// 		let node = TreeNode {
// 			id: root,
// 			version,
// 			name,
// 			children: children_nodes,
// 			signature,
// 		};
// 		let rc = Arc::new(node);
// 		self.runtime.tree.insert(root, Arc::clone(&rc));
// 		rc
// 	}
// 	pub fn compute_signature(&self, id: u64, version: u64, child_sigs: &[u64]) -> u64 {
// 		let mut h = version.wrapping_mul(31).wrapping_add(id);
// 		for s in child_sigs {
// 			h = h.wrapping_mul(31) ^ s;
// 		}
// 		h
// 	}
// 	pub fn get_compiled(&self, id: u64) -> Option<&CompiledNode> {
// 		let compiled = self.runtime.compiled.get(&id)?;
// 		let inode = self.store.inodes.get(&id)?;
// 		if compiled.source_version == inode.version {
// 			Some(compiled)
// 		} else {
// 			None
// 		}
// 	}
// }
// impl Vfs {
// 	pub fn export(&self) -> Vec<u8> {
// 		let mut out = Vec::new();
// 		out.extend_from_slice(&self.store.next_id.to_le_bytes());
// 		let count = self.store.inodes.len() as u64;
// 		out.extend_from_slice(&count.to_le_bytes());
// 		for (id, inode) in &self.store.inodes {
// 			out.extend_from_slice(&id.to_le_bytes());
// 			out.extend_from_slice(&inode.version.to_le_bytes());
// 			let name_bytes = inode.name.as_bytes();
// 			let name_len = name_bytes.len() as u64;
// 			out.extend_from_slice(&name_len.to_le_bytes());
// 			out.extend_from_slice(name_bytes);
// 			match &inode.kind {
// 				InodeKind::File(data) => {
// 					out.push(0);
// 					let len = data.len() as u64;
// 					out.extend_from_slice(&len.to_le_bytes());
// 					out.extend_from_slice(data);
// 				}
// 				InodeKind::Directory(children) => {
// 					out.push(1);
// 					let len = children.len() as u64;
// 					out.extend_from_slice(&len.to_le_bytes());
// 					for c in children {
// 						out.extend_from_slice(&c.to_le_bytes());
// 					}
// 				}
// 			}
// 		}
// 		out
// 	}
// 	pub fn import(&mut self, data: &[u8]) {
// 		self.store.inodes.clear();
// 		self.runtime.tree.clear();
// 		self.runtime.compiled.clear();
// 		self.runtime.dirty.clear();
// 		let mut i = 0;
// 		let read_u64 = |i: &mut usize| -> u64 {
// 			let mut buf = [0u8; 8];
// 			buf.copy_from_slice(&data[*i..*i + 8]);
// 			*i += 8;
// 			u64::from_le_bytes(buf)
// 		};
// 		self.store.next_id = read_u64(&mut i);
// 		let inode_count = read_u64(&mut i);
// 		for _ in 0..inode_count {
// 			let id = read_u64(&mut i);
// 			let version = read_u64(&mut i);
// 			let mut len_buf = [0u8; 8];
// 			len_buf.copy_from_slice(&data[i..i + 8]);
// 			i += 8;
// 			let name_len = u64::from_le_bytes(len_buf) as usize;
// 			let name = String::from_utf8(data[i..i + name_len].to_vec()).unwrap();
// 			i += name_len;
// 			let kind_tag = data[i];
// 			i += 1;
// 			let inode = match kind_tag {
// 				0 => {
// 					let len = read_u64(&mut i) as usize;
// 					let file = data[i..i + len].to_vec();
// 					i += len;
// 					Inode {
// 						version,
// 						meta: Meta {
// 							created_at: 0,
// 							modified_at: 0,
// 							size: file.len() as u64,
// 						},
// 						// name,
// 						kind: InodeKind::File(file),
// 					}
// 				}
// 				1 => {
// 					let len = read_u64(&mut i) as usize;
// 					let mut children = Vec::new();
// 					for _ in 0..len {
// 						children.push(read_u64(&mut i));
// 					}
// 					Inode {
// 						version,
// 						meta: Meta {
// 							created_at: 0,
// 							modified_at: 0,
// 							size: 0,
// 						},
// 						// name,
// 						kind: InodeKind::Directory(children),
// 					}
// 				}
// 				_ => panic!("invalid inode kind"),
// 			};
// 			self.store.inodes.insert(id, inode);
// 		}
// 	}
// 	pub fn export_json(&self) -> String {
// 		format!(
// 			r#"{{
//                 "next_id": {},
//                 "inode_count": {}
//             }}"#,
// 			self.store.next_id,
// 			self.store.inodes.len()
// 		)
// 	}
// 	pub fn import_json(&mut self, _json: &str) {
// 		// intentionally left minimal until format is locked
// 		// real version should mirror binary import logic
// 	}
// }
#[derive(Clone, Debug, Default)]
pub struct Store {
	pub inodes: HashMap<u64, Inode>,
	pub next_id: u64,
}
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct RuntimeCache {
	pub tree: HashMap<u64, Arc<TreeNode>>,
	pub compiled: HashMap<u64, CompiledNode>,
	pub dirty: HashSet<u64>,
}
// pub struct Runtime {
// 	pub tree: HashMap<u64, Vec<u64>>,
// 	pub parents: HashMap<u64, u64>,
// 	pub compiled: HashMap<u64, CompiledNode>,
// 	pub dirty: HashSet<u64>,
// }

// ┌─────────────────┐
// │     Inode       │
// │ identity/kind   │
// │ timestamps/size │
// └────────┬────────┘
//          │
//          ▼
// ┌─────────────────┐
// │      File       │
// │ content access  │
// │ path/name       │
// └────────┬────────┘
//          │
// ┌────────┴────────┐
// ▼                 ▼
// ┌────────────┐    ┌───────────────┐
// │   Image    │    │     Media     │
// │  metadata  │    │    metadata   │
// └────────────┘    └───────────────┘
// pub struct Fs {
// 	root: Uuid,
// 	next_inode_id: Uuid,
// 	inodes: HashMap<Uuid, Inode>,
// }
// impl Fs {
// 	fn alloc_inode_id(&mut self) -> Uuid {
// 		let id = self.next_inode_id;
// 		self.next_inode_id = self.next_inode_id.next();
// 		id
// 	}
// 	fn create_inode(&mut self, kind: InodeKind) -> Inode {
// 		let id = self.alloc_inode_id();
// 		let inode = Inode {
// 			id,
// 			version: 1,
// 			kind,
// 			meta: Meta::default(),
// 		};
// 		self.inodes.insert(id, inode);
// 		inode
// 	}
// 	pub fn create_file(&mut self, storage: StorageRef) -> File {
// 		let inode = self.create_inode(InodeKind::File);
// 		File { inode, storage }
// 	}
// 	pub fn create_directory(&mut self) -> Directory {
// 		let inode = self.create_inode(InodeKind::Directory);
// 		Directory {
// 			entries: HashMap::default(),
// 		}
// 	}
// 	pub fn new() -> Self {
// 		let uid = Uuid::new_v4();
// 		let root = Inode {
// 			version: 1,
// 			id: uid,
// 			meta: Meta::default(),
// 			kind: InodeKind::Directory,
// 		};
// 		let mut inodes = HashMap::new();
// 		inodes.insert(root.id, root.clone());
// 		Self {
// 			inodes,
// 			root: root.id,
// 			next_inode_id: Uuid::new_v4(),
// 		}
// 	}
// }

// #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
// pub struct InodeId(u64);
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct MemoryId(u64);
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct RemoteId(u64);
// impl InodeId {
// 	fn new(value: u64) -> Self {
// 		Self(value)
// 	}
// 	fn first() -> Self {
// 		Self(1)
// 	}
// 	fn next(self) -> Self {
// 		Self(self.0 + 1)
// 	}
// }
#[derive(Clone, Copy, Debug, Deserialize, Hash, Serialize)]
pub struct Inode {
	pub id: Uuid,
	pub version: u64,
	pub kind: InodeKind,
	pub meta: Meta,
}

pub trait Storage {
	fn read(&self) -> Vec<u8>;
	fn write(&mut self, data: &[u8]);
	fn exists(&self) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct InodeStore {
	inodes: HashMap<Uuid, Inode>,
	next_id: Uuid,
}
impl InodeStore {
	pub fn new() -> Self {
		Self {
			inodes: HashMap::new(),
			next_id: Uuid::new_v4(),
		}
	}

	// fn alloc_id(&mut self) -> Uuid {
	// 	let id = self.next_id;
	// 	self.next_id = self.next_id.next();
	// 	id
	// }

	pub fn create(&mut self, kind: InodeKind) -> Uuid {
		let id = Uuid::new_v4();

		let inode = Inode {
			id,
			version: 1,
			kind,
			meta: Meta::default(),
		};

		self.inodes.insert(id, inode);

		id
	}

	pub fn get(&self, id: Uuid) -> Option<&Inode> {
		self.inodes.get(&id)
	}

	pub fn get_mut(&mut self, id: Uuid) -> Option<&mut Inode> {
		self.inodes.get_mut(&id)
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum InodeKind {
	File,
	Directory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Meta {
	pub created_at: u64,
	pub modified_at: u64,
	pub size: u64,
}
pub struct File {
	pub inode: Inode,
	pub storage: StorageRef,
}

pub enum StorageRef {
	Memory(MemoryId),
	Local(PathBuf),
	Remote(RemoteId),
}
pub struct Directory {
	pub entries: HashMap<String, Uuid>,
}

#[derive(Clone, Debug)]
pub struct DirEntry {
	pub name: String,
	pub inode: Uuid,
}
pub struct FileView<'a> {
	data: &'a [u8],
}
pub struct DependencyGraph {
	pub dependents: HashMap<u64, HashSet<u64>>,
	pub dependencies: HashMap<u64, HashSet<u64>>,
}
pub struct SourceAnalysis {
	pub imports: Vec<String>,
	pub exports: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct TreeNode {
	pub id: u64,
	pub version: u64,
	pub name: String,
	pub children: Vec<TreeNode>,
	pub signature: u64,
}
#[derive(Debug, Clone)]
pub enum Payload {
	Module { source: Vec<u8> },
	Ast { nodes: String },
	DirectoryIndex { children: Vec<u64> },
}
impl DependencyGraph {
	pub fn contains_cycle(&self) -> bool {
		let mut visited = HashSet::new();
		let mut rec_stack = HashSet::new();
		pub fn has_cycle(
			id: u64,
			deps: &HashMap<u64, HashSet<u64>>,
			visited: &mut HashSet<u64>,
			rec_stack: &mut HashSet<u64>,
		) -> bool {
			visited.insert(id);
			rec_stack.insert(id);
			if let Some(targets) = deps.get(&id) {
				for &target in targets {
					if !visited.contains(&target) && has_cycle(target, deps, visited, rec_stack) {
						return true;
					} else if rec_stack.contains(&target) {
						return true;
					}
				}
			}
			rec_stack.remove(&id);
			false
		}
		for &id in self.dependencies.keys() {
			if !visited.contains(&id) && has_cycle(id, &self.dependencies, &mut visited, &mut rec_stack) {
				return true;
			}
		}
		false
	}
}
#[derive(Debug, Clone)]
pub struct CompiledNode {
	pub id: u64,
	pub source_version: u64,
	pub payload: Payload,
	pub imports: Vec<String>,
	pub exports: Vec<String>,
}
impl CompiledNode {
	pub fn new(
		id: u64,
		source_version: u64,
		raw_source: Vec<u8>,
		imports: Vec<String>,
		exports: Vec<String>,
	) -> Self {
		let payload = Payload::Module { source: raw_source };
		Self {
			id,
			source_version,
			payload,
			imports,
			exports,
		}
	}
}
#[derive(Debug)]
pub enum VfsError {
	InodeNotFound(u64),
	IoError(std::io::Error),
	SyntaxError(String),
	DependencyCycle(u64),
}
type VfsResult<T> = Result<T, VfsError>;
impl std::fmt::Display for Store {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Store")?;
		writeln!(f, "  \tinodes: {}", self.inodes.len())?;
		writeln!(f, "  \tnext_id: {}", self.next_id)?;
		Ok(())
	}
}
impl std::fmt::Display for RuntimeCache {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "RuntimeCache")?;
		writeln!(f, "  \ttree: {}", self.tree.len())?;
		writeln!(f, "  \tcompiled: {}", self.compiled.len())?;
		writeln!(f, "  \tdirty: {}", self.dirty.len())?;
		Ok(())
	}
}
// impl std::fmt::Display for Vfs {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		writeln!(f, "Vfs")?;
// 		// writeln!(f, "  \t{}", self.store)?;
// 		// writeln!(f, "  \t{}", self.runtime)?;
// 		Ok(())
// 	}
// }
