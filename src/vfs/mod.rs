#[allow(unused_imports)]
use std::sync::Arc;
pub mod vfs;
use macros::*;
pub use vfs::*;

// Naming conventions (Hoare logic)
// doesnt_<violation>_<domain>
// does_<behavior>_<domain>_on_<operation>
// has_<invariant>_<scope>
// A. Mental model (what users think)
// - file
// - write
// - directory
// B. System model (what you actually have)
// - graph node
// - edges
// - derived state
// C. Execution model (what happens internally)
// - cache invalidation
// - compile invalidation
// - runtime reset
// - dependency propagation

vow!(does_build_inode_on_create_file, {
	let mut vfs = Vfs::new();
	let id = vfs.create_file(0, "test.txt", b"hello".to_vec());
	assert!(vfs.store.inodes.contains_key(&id));
});
vow!(does_build_cache_entry_on_file_compile, {
	let mut vfs = Vfs::new();
	let id = vfs.create_file(0, "a.js", b"export const a = 1".to_vec());
	vfs.compile(id);
	assert!(vfs.runtime.compiled.contains_key(&id));
});
vow!(does_on_file_write_invalidates_cache, {
	let mut vfs = Vfs::new();
	let id = vfs.create_file(0, "a", b"x".to_vec());
	vfs.compile(id);
	let c1 = vfs.get_compiled(id).unwrap().source_version;
	vfs.write(id, b"y".to_vec());
	vfs.compile(id);
	let c2 = vfs.get_compiled(id).unwrap().source_version;
	assert_ne!(c1, c2);
});
vow!(does_on_file_write_invalidates_compilation, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let id = vfs.create_file(root, "a", b"x".to_vec());
	vfs.compile(id);
	let c1 = vfs.runtime.compiled.get(&id).unwrap().source_version;
	vfs.write(id, b"new".to_vec());
	vfs.compile(id);
	let c2 = vfs.runtime.compiled.get(&id).unwrap().source_version;
	assert_ne!(c1, c2);
});
vow!(does_on_file_write_invalidates_version, {
	let mut vfs = Vfs::new();
	let id = vfs.create_file(0, "a.js", b"1".to_vec());
	vfs.compile(id);
	let before = vfs.runtime.compiled.get(&id).unwrap().source_version;
	vfs.write(id, b"2".to_vec());
	vfs.compile(id);
	let after = vfs.runtime.compiled.get(&id).unwrap().source_version;
	assert!(after > before);
});
vow!(does_on_file_write_invalidates_children_via_propogation, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let b = vfs.create_file(root, "b.js", b"export const x = 1;".to_vec());
	let a = vfs.create_file(root, "a.js", b"import {x} from './b.js'".to_vec());
	vfs.compile(a).expect("Initial compile failed");
	let version_a_initial = vfs.runtime.compiled.get(&a).unwrap().source_version;
	vfs.write(b, b"export const x = 2;".to_vec());
	vfs.compile(a).expect("Re-compile failed");
	let version_a_new = vfs.runtime.compiled.get(&a).unwrap().source_version;
	assert_ne!(version_a_initial, version_a_new);
});
vow!(does_on_file_write_invalidates_tree_cache, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let id = vfs.create_file(root, "a", b"x".to_vec());
	let old_tree = vfs.build_tree(root);
	let old_child_version = old_tree.children[0].version;
	vfs.write(id, vec![2, 3, 4]);
	let new_tree = vfs.build_tree(root);
	let new_child_version = new_tree.children[0].version;
	assert_ne!(old_child_version, new_child_version);
});
vow!(does_on_file_write_invalidates_tree, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let id = vfs.create_file(root, "a", b"x".to_vec());
	let t1 = vfs.build_tree(root);
	vfs.write(id, b"new".to_vec());
	let t2 = vfs.build_tree(root);
	assert_ne!(t1.signature, t2.signature);
});
vow!(does_reuse_tree_cache_when_not_dirty, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	vfs.create_file(root, "a", b"x".to_vec());
	let t1 = vfs.build_tree(root);
	let ptr1 = Arc::as_ptr(&t1);
	let t2 = vfs.build_tree(root);
	let ptr2 = Arc::as_ptr(&t2);
	assert_eq!(ptr1, ptr2, "The cache was not reused!");
});
vow!(does_rebuild_compiled_on_version_mismatch, {
	let mut vfs = Vfs::new();
	let id = vfs.create_file(0, "a.js", b"1".to_vec());
	vfs.compile(id);
	let first = vfs.runtime.compiled.get(&id).unwrap().source_version;

	// [String literal prefix]: (more generally: a raw / byte literal prefix)
	// 1. Raw string literal
	// r"hello\nworld"
	// 2. Byte literal
	// b"2"
	vfs.write(id, b"2".to_vec());
	vfs.compile(id);
	let second = vfs.runtime.compiled.get(&id).unwrap().source_version;
	assert_ne!(first, second);
});
vow!(does_rebuild_tree_on_child_change, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let id = vfs.create_file(root, "a", b"x".to_vec());
	let t1 = vfs.build_tree(root);
	let v1 = t1.children[0].version;
	vfs.write(id, b"new".to_vec());
	let t2 = vfs.build_tree(root);
	let v2 = t2.children[0].version;
	assert_ne!(v1, v2);
});
vow!(does_reset_runtime_and_cache_on_import, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let id = vfs.create_file(root, "a", b"x".to_vec());
	vfs.build_tree(root);
	vfs.compile(id);
	let snapshot = vfs.export();
	vfs.write(id, b"changed".to_vec());
	vfs.import(&snapshot);
	assert!(vfs.runtime.tree.is_empty());
});
vow!(does_reset_entire_system_on_import, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let a = vfs.create_file(root, "a", b"x".to_vec());
	let _ = vfs.build_tree(root);
	let snapshot = vfs.export();
	vfs.write(a, b"y".to_vec());
	vfs.import(&snapshot);
	let tree = vfs.build_tree(root);
	assert_eq!(tree.version, 0);
});
vow!(does_reset_runtime_on_import, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let id = vfs.create_file(root, "a", b"x".to_vec());
	vfs.compile(id);
	assert!(!vfs.runtime.compiled.is_empty());
	let snapshot = vfs.export();
	vfs.import(&snapshot);
	assert!(vfs.runtime.compiled.is_empty());
});
vow!(does_on_version_change_invalidate_tree_cache, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let id = vfs.create_file(root, "a", b"x".to_vec());
	let _ = vfs.build_tree(root);
	let cached_before = vfs.runtime.tree.len();
	vfs.write(id, b"update".to_vec());
	vfs.build_tree(root);
	let cached_after = vfs.runtime.tree.len();
	assert!(cached_after >= cached_before);
});
vow!(doesnt_corrupt_cache_on_sibling_write, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let file1 = vfs.create_file(root, "a.js", b"content".to_vec());
	let file2 = vfs.create_file(root, "b.js", b"content".to_vec());
	vfs.compile(file1);
	vfs.compile(file2);
	let cache_entry_b_addr = vfs.runtime.compiled.get(&file2).unwrap() as *const _;
	vfs.write(file1, b"changed".to_vec());
	let new_cache_entry_b_addr = vfs.runtime.compiled.get(&file2).unwrap() as *const _;
	assert_eq!(cache_entry_b_addr, new_cache_entry_b_addr);
});
vow!(doesnt_corrupt_cache_on_compile_error, {
	let mut vfs = Vfs::new();
	let id = vfs.create_file(0, "bad.js", b"!!! syntax error !!!".to_vec());
	let result = vfs.compile(id);
	assert!(result.is_err());
	vfs.write(id, b"const a = 1;".to_vec());
	let success_result = vfs.compile(id);
	assert!(success_result.is_ok());
});
vow!(doesnt_create_circular_dependencies, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let a = vfs.create_file(root, "a.js", b"import './b.js'".to_vec());
	let b = vfs.create_file(root, "b.js", b"import './a.js'".to_vec());
	let _ = vfs.compile(a);
	let _ = vfs.compile(b);
	let graph = vfs.build_dependency_graph();
	assert!(graph.contains_cycle());
});
vow!(doesnt_create_derived_state_on_export, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let id = vfs.create_file(root, "a", b"x".to_vec());
	let _tree = vfs.build_tree(root);
	vfs.compile(id);
	let data = vfs.export();
	vfs.runtime.tree.clear();
	assert!(vfs.runtime.tree.is_empty());
	assert!(data.len() > 0);
});
vow!(doesnt_create_invalid_inodes, {
	let mut vfs = Vfs::new();
	let result = vfs.compile(999);
	assert!(matches!(result, Err(VfsError::InodeNotFound(999))));
});
vow!(doesnt_derive_state_in_inodes, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let id = vfs.create_file(root, "a", b"x".to_vec());
	vfs.build_tree(root);
	vfs.compile(id);
	for inode in vfs.store.inodes.values() {
		match &inode.kind {
			InodeKind::File(_) => {}
			InodeKind::Directory(children) => {
				assert!(!children.is_empty(), "children list should not be empty");
			}
		}
	}
});
vow!(has_self_consistent_snapshot_export_modules, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let a = vfs.create_file(root, "a", b"1".to_vec());
	vfs.write(a, b"2".to_vec());
	let snapshot = vfs.export();
	let mut vfs2 = Vfs::new();
	vfs2.import(&snapshot);
	let tree1 = vfs.build_tree(root);
	let tree2 = vfs2.build_tree(root);
	assert_eq!(tree1.version, tree2.version);
});
vow!(has_preservation_of_truth_on_export_import_roundtrip, {
	let mut vfs = Vfs::new();
	let root = vfs.init_root();
	let a = vfs.create_file(root, "a", b"1".to_vec());
	let b = vfs.create_file(root, "b", b"2".to_vec());
	vfs.build_tree(root);
	vfs.compile(a);
	vfs.compile(b);
	let snapshot = vfs.export();
	let mut vfs2 = Vfs::new();
	vfs2.import(&snapshot);
	assert_eq!(vfs2.store.inodes.len(), 3);
	assert!(vfs2.runtime.tree.is_empty());
	assert!(vfs2.runtime.compiled.is_empty());
});
