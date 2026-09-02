#![allow(warnings)]
use jsonc_parser::{
	ParseOptions,
	cst::{CstLeafNode, CstNode, CstRootNode},
};
#[derive(Debug)]
struct SourceEntry {
	sort_key: Option<String>,
	leading: String,
	value: String,
	trailing: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortOrder {
	Asc,
	Desc,
}
impl SortOrder {
	fn compare(&self, a: &str, b: &str) -> std::cmp::Ordering {
		match self {
			Self::Asc => a.cmp(b),
			Self::Desc => b.cmp(a),
		}
	}
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MissingSort {
	First,
	Last,
}
fn sort_entries(
	mut entries: Vec<SourceEntry>,
	order: SortOrder,
	missing: MissingSort,
) -> Vec<SourceEntry> {
	entries.sort_by(|a, b| match (&a.sort_key, &b.sort_key) {
		(Some(a), Some(b)) => order.compare(a, b),
		(None, None) => std::cmp::Ordering::Equal,
		(None, Some(_)) => match missing {
			MissingSort::First => std::cmp::Ordering::Less,
			MissingSort::Last => std::cmp::Ordering::Greater,
		},
		(Some(_), None) => match missing {
			MissingSort::First => std::cmp::Ordering::Greater,
			MissingSort::Last => std::cmp::Ordering::Less,
		},
	});
	entries
}
fn render_entries(entries: &[SourceEntry]) -> String {
	let mut output = String::new();
	for entry in entries {
		output.push_str(&entry.leading);
		output.push_str(&entry.value);
		output.push_str(&entry.trailing);
	}
	output
}
fn render_array(source: &str, entries: &[SourceEntry]) -> String {
	let array_start = source.find('[').expect("array must have '['");
	let array_end = source.rfind(']').expect("array must have ']'");
	let prefix = &source[..=array_start];
	let suffix = &source[array_end..];
	let mut output = String::new();
	output.push_str(prefix);
	output.push_str(&render_entries(entries));
	output.push_str(suffix);
	output
}
fn extract_array_entries(
	children: &[CstNode],
	elements: &[CstNode],
	field: &str,
) -> Vec<SourceEntry> {
	let chunks = source_chunks(children, elements);
	elements
		.iter()
		.zip(chunks)
		.map(|(element, (leading, value, trailing))| SourceEntry {
			sort_key: node_sort_value(element, field),
			leading,
			value,
			trailing,
		})
		.collect()
}
fn sort_object(source: &str, order: SortOrder) -> String {
	let root = CstRootNode::parse(source, &ParseOptions::default()).expect("invalid JSONC");
	let object = root.object_value().expect("expected JSON object");
	let properties = object.properties();
	if properties.len() <= 1 {
		return source.to_string();
	}
	#[derive(Debug)]
	struct Entry {
		key: String,
		text: String,
	}
	let entries: Vec<Entry> = properties
		.iter()
		.map(|property| {
			let key = property
				.name()
				.and_then(|name| name.decoded_value().ok())
				.expect("property must have a name");
			Entry {
				key,
				text: property.to_string(),
			}
		})
		.collect();
	let mut sorted = entries;
	sorted.sort_by(|a, b| order.compare(&a.key, &b.key));
	/*
	 * CstObjectProp::to_string() gives us the property itself:
	 *
	 *     "foo": 123
	 *
	 * but NOT the comma / whitespace separating it from
	 * neighboring properties.
	 *
	 * We therefore extract the formatting between the original
	 * properties and reuse that formatting for the sorted output.
	 */
	let original_text: Vec<String> = properties.iter().map(|p| p.to_string()).collect();
	let first = &original_text[0];
	let last = &original_text[original_text.len() - 1];
	let first_start = source.find(first).expect("first property must exist");
	let last_start = source.rfind(last).expect("last property must exist");
	let last_end = last_start + last.len();
	let prefix = &source[..first_start];
	let suffix = &source[last_end..];
	/*
	 * Extract the separator between properties.
	 *
	 * Example:
	 *
	 *     "a": 1,
	 *
	 *     "b": 2
	 *
	 * separator becomes:
	 *
	 *     ",\n\n    "
	 */
	let mut separators = Vec::new();
	for pair in original_text.windows(2) {
		let left = &pair[0];
		let right = &pair[1];
		let left_start = source.find(left).expect("property must exist");
		let left_end = left_start + left.len();
		let right_start = source[left_end..]
			.find(right)
			.map(|offset| left_end + offset)
			.expect("next property must exist");
		separators.push(source[left_end..right_start].to_string());
	}
	/*
	 * Most normal JSON/JSONC objects use the same separator.
	 *
	 * Preserve the first separator as the formatting template.
	 */
	let separator = separators
		.first()
		.cloned()
		.unwrap_or_else(|| ",".to_string());
	let mut output = String::new();
	output.push_str(prefix);
	for (index, entry) in sorted.iter().enumerate() {
		if index > 0 {
			output.push_str(&separator);
		}
		output.push_str(&entry.text);
	}
	output.push_str(suffix);
	output
}
fn extract_field(source: &str, field: &str) -> Option<String> {
	let root = CstRootNode::parse(source, &ParseOptions::default()).ok()?;
	let object = root.object_value()?;
	let property = object.get(field)?;
	let value = property.value()?;
	match value {
		CstNode::Leaf(leaf) => Some(leaf.to_string().trim_matches('"').to_string()),
		_ => None,
	}
}
fn extract_array_layout(
	source: &str,
	children: &[CstNode],
	elements: &[CstNode],
	field: &str,
) -> (String, Vec<SourceEntry>, Vec<String>, String) {
	let first = elements.first().unwrap();
	let last = elements.last().unwrap();
	let first_index = first.child_index();
	let last_index = last.child_index();
	let array_start = source.find('[').expect("array must have '['");
	let array_end = source.rfind(']').expect("array must have ']'");
	/*
	 * Everything between '[' and the first actual element.
	 *
	 * Example:
	 *
	 * [
	 *     <--- prefix
	 *     { ... }
	 */
	let first_text = first.to_string();
	let first_start = source.find(&first_text).expect("first element must exist");
	let prefix = source[array_start + 1..first_start].to_string();
	/*
	 * Extract the actual element values.
	 *
	 * Separators are deliberately NOT part of the entries.
	 */
	let mut entries = Vec::with_capacity(elements.len());
	for element in elements {
		entries.push(SourceEntry {
			sort_key: node_sort_value(element, field),
			leading: String::new(),
			value: element.to_string(),
			trailing: String::new(),
		});
	}
	/*
	 * Extract formatting between elements.
	 *
	 * These belong to array positions, not to the objects.
	 */
	let mut separators = Vec::with_capacity(elements.len().saturating_sub(1));
	for pair in elements.windows(2) {
		let left = &pair[0];
		let right = &pair[1];
		let left_text = left.to_string();
		let right_text = right.to_string();
		let left_start = source.find(&left_text).expect("left element must exist");
		let left_end = left_start + left_text.len();
		let right_start = source[left_end..]
			.find(&right_text)
			.map(|offset| left_end + offset)
			.expect("right element must exist");
		separators.push(source[left_end..right_start].to_string());
	}
	/*
	 * Everything after the last element belongs to the
	 * array itself.
	 */
	let last_text = last.to_string();
	let last_start = source.rfind(&last_text).expect("last element must exist");
	let last_end = last_start + last_text.len();
	let suffix = source[last_end..array_end].to_string();
	let _ = children;
	let _ = first_index;
	let _ = last_index;
	(prefix, entries, separators, suffix)
}
fn render_array_layout(
	prefix: &str,
	entries: &[SourceEntry],
	separators: &[String],
	suffix: &str,
) -> String {
	let mut output = String::new();
	output.push('[');
	output.push_str(prefix);
	for (index, entry) in entries.iter().enumerate() {
		output.push_str(&entry.value);
		if let Some(separator) = separators.get(index) {
			output.push_str(separator);
		}
	}
	output.push_str(suffix);
	output.push(']');
	output
}
fn sort_array_with_missing(
	source: &str,
	field: &str,
	order: SortOrder,
	missing: MissingSort,
) -> String {
	let root = CstRootNode::parse(source, &ParseOptions::default()).expect("invalid JSONC");
	let array = root.array_value().expect("expected JSON array");
	let elements = array.elements();
	if elements.len() <= 1 {
		return source.to_string();
	}
	let children = array.children();
	let (prefix, entries, separators, suffix) =
		extract_array_layout(source, &children, &elements, field);
	let entries = sort_entries(entries, order, missing);
	render_array_layout(&prefix, &entries, &separators, &suffix)
}
fn sort_array(source: &str, field: &str, order: SortOrder) -> String {
	sort_array_with_missing(source, field, order, MissingSort::Last)
}
fn source_chunks(children: &[CstNode], entries: &[CstNode]) -> Vec<(String, String, String)> {
	let mut result = Vec::new();
	for (i, entry) in entries.iter().enumerate() {
		let start = entry.child_index();
		let end = entries
			.get(i + 1)
			.map(|next| next.child_index())
			.unwrap_or(children.len());
		let mut leading = String::new();
		let mut value = String::new();
		let mut trailing = String::new();
		for child in &children[start..end] {
			if child.is_trivia() {
				leading.push_str(&child.to_string());
			} else if child.is_token() && child.token_char() == Some(',') {
				trailing.push_str(&child.to_string());
			} else {
				value.push_str(&child.to_string());
			}
		}
		result.push((leading, value, trailing));
	}
	result
}
fn node_sort_value(node: &CstNode, field: &str) -> Option<String> {
	let object = node.as_object()?;
	let property = object.get(field)?;
	let value = property.value()?;
	match value {
		CstNode::Leaf(CstLeafNode::StringLit(value)) => value.decoded_value().ok(),
		CstNode::Leaf(CstLeafNode::WordLit(value)) => Some(value.to_string()),
		CstNode::Leaf(CstLeafNode::NumberLit(value)) => Some(value.to_string()),
		CstNode::Leaf(CstLeafNode::BooleanLit(value)) => Some(value.to_string()),
		CstNode::Leaf(CstLeafNode::NullKeyword(value)) => Some(value.to_string()),
		CstNode::Leaf(
			CstLeafNode::Token(_)
			| CstLeafNode::Whitespace(_)
			| CstLeafNode::Newline(_)
			| CstLeafNode::Comment(_),
		) => None,
		CstNode::Container(_) => None,
	}
}
fn sort_recursive(source: &str, order: SortOrder) -> String {
	let root = CstRootNode::parse(source, &ParseOptions::default()).expect("invalid JSONC");
	if root.object_value().is_some() {
		return sort_recursive_object(source, order);
	}
	if root.array_value().is_some() {
		return sort_recursive_array(source, order);
	}
	source.to_string()
}
fn sort_recursive_object(source: &str, order: SortOrder) -> String {
	let root = CstRootNode::parse(source, &ParseOptions::default()).expect("invalid JSONC");
	let object = match root.object_value() {
		Some(object) => object,
		None => return source.to_string(),
	};
	let properties = object.properties();
	if properties.is_empty() {
		return source.to_string();
	}
	/*
	 * First recursively transform nested values.
	 */
	let mut result = source.to_string();
	for property in properties.iter() {
		let value = match property.value() {
			Some(value) => value,
			None => continue,
		};
		let value_text = value.to_string();
		if value.as_object().is_some() || value.as_array().is_some() {
			let sorted_value = sort_recursive(&value_text, order);
			result = result.replacen(&value_text, &sorted_value, 1);
		}
	}
	/*
	 * Now sort this object's own properties.
	 */
	sort_object(&result, order)
}
fn sort_recursive_array(source: &str, order: SortOrder) -> String {
	let root = CstRootNode::parse(source, &ParseOptions::default()).expect("invalid JSONC");
	let array = match root.array_value() {
		Some(array) => array,
		None => return source.to_string(),
	};
	let elements = array.elements();
	if elements.is_empty() {
		return source.to_string();
	}
	/*
	 * Preserve array element order.
	 *
	 * We only recurse into each element.
	 */
	let mut result = source.to_string();
	for element in elements.iter() {
		if element.as_object().is_none() && element.as_array().is_none() {
			continue;
		}
		let element_text = element.to_string();
		let sorted_element = sort_recursive(&element_text, order);
		result = result.replacen(&element_text, &sorted_element, 1);
	}
	result
}
fn sort_zed_bindings(source: &str, order: SortOrder) -> String {
	sort_recursive(source, order)
}
fn sort_zed_unbinds(source: &str, order: SortOrder) -> String {
	sort_recursive(source, order)
}
fn sort_zed_contexts(source: &str, order: SortOrder) -> String {
	sort_array_with_missing(source, "context", order, MissingSort::First)
}
fn move_zed_context_first(source: &str) -> String {
	let root = CstRootNode::parse(source, &ParseOptions::default()).expect("invalid JSONC");
	let array = match root.array_value() {
		Some(array) => array,
		None => return source.to_string(),
	};
	let mut result = source.to_string();
	for element in array.elements() {
		if element.as_object().is_none() {
			continue;
		}
		let element_text = element.to_string();
		let moved = move_object_key_first(&element_text, "context");
		result = result.replacen(&element_text, &moved, 1);
	}
	result
}
fn sort_zed_keymap(source: &str, order: SortOrder) -> String {
	// Sort keymap entries: contextless first, then context alphabetically.
	let source = sort_array_with_missing(source, "context", order, MissingSort::First);
	// Recursively alphabetize all objects.
	let source = sort_recursive(&source, order);
	// Zed wants `context` first in each keymap entry.
	move_object_key_first(&source, "context")
}
fn main() {}
#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn sorts_top_level_keys_ascending() {
		let input = r#"{
        "z": 1,
        "a": 2,
        "m": 3
    }"#;
		let expected = r#"{
        "a": 2,
        "m": 3,
        "z": 1
    }"#;
		assert_eq!(sort_object(input, SortOrder::Asc), expected);
	}
	#[test]
	fn sorts_top_level_keys_descending() {
		let input = r#"{
        "a": 1,
        "z": 2,
        "m": 3
    }"#;
		let expected = r#"{
        "z": 2,
        "m": 3,
        "a": 1
    }"#;
		assert_eq!(sort_object(input, SortOrder::Desc), expected);
	}
	#[test]
	fn zed_settings_preserves_comments_when_sorting_recursively() {
		let input = r#"{
        // Formatting
        "format_on_save": "on",
        // Language configuration
        "languages": {
            // Rust
            "Rust": {
                // Rust formatting
                "format_on_save": "on",
                "use_on_type_format": true
            },
            // TypeScript
            "TypeScript": {
                "use_on_type_format": true,
                "format_on_save": "on"
            }
        },
        // Autosave
        "autosave": "on_focus_change"
    }"#;
		let expected = r#"{
        // Autosave
        "autosave": "on_focus_change",
        // Formatting
        "format_on_save": "on",
        // Language configuration
        "languages": {
            // Rust
            "Rust": {
                // Rust formatting
                "format_on_save": "on",
                "use_on_type_format": true
            },
            // TypeScript
            "TypeScript": {
                "format_on_save": "on",
                "use_on_type_format": true
            }
        }
    }"#;
		assert_eq!(sort_recursive(input, SortOrder::Asc), expected);
	}
	// #[test]
	// fn sorts_keybindings_by_key_ascending() {
	// 	let input = r#"[
	//        {
	//            "key": "up",
	//            "command": "list.focusUp"
	//        },
	//        {
	//            "key": "tab",
	//            "command": "editor.tab"
	//        },
	//        {
	//            "key": "ctrl+a",
	//            "command": "selectAll"
	//        }
	//    ]"#;
	// 	let expected = r#"[
	//        {
	//            "key": "ctrl+a",
	//            "command": "selectAll"
	//        },
	//        {
	//            "key": "tab",
	//            "command": "editor.tab"
	//        },
	//        {
	//            "key": "up",
	//            "command": "list.focusUp"
	//        }
	//    ]"#;
	// 	assert_eq!(sort_array(input, "key", SortOrder::Asc), expected);
	// }
	// #[test]
	// fn sorts_keybindings_by_key_descending() {
	// 	let input = r#"[
	//        {
	//            "key": "up",
	//            "command": "list.focusUp"
	//        },
	//        {
	//            "key": "tab",
	//            "command": "editor.tab"
	//        },
	//        {
	//            "key": "ctrl+a",
	//            "command": "selectAll"
	//        }
	//    ]"#;
	// 	let expected = r#"[
	//        {
	//            "key": "up",
	//            "command": "list.focusUp"
	//        },
	//        {
	//            "key": "tab",
	//            "command": "editor.tab"
	//        },
	//        {
	//            "key": "ctrl+a",
	//            "command": "selectAll"
	//        }
	//    ]"#;
	// 	assert_eq!(sort_array(input, "key", SortOrder::Desc), expected);
	// }
	// #[test]
	// fn sorts_keybindings_by_command_ascending() {
	// 	let input = r#"[
	//        {
	//            "key": "up",
	//            "command": "list.focusUp"
	//        },
	//        {
	//            "key": "tab",
	//            "command": "editor.tab"
	//        },
	//        {
	//            "key": "ctrl+a",
	//            "command": "selectAll"
	//        }
	//    ]"#;
	// 	let expected = r#"[
	//        {
	//            "key": "ctrl+a",
	//            "command": "selectAll"
	//        },
	//        {
	//            "key": "tab",
	//            "command": "editor.tab"
	//        },
	//        {
	//            "key": "up",
	//            "command": "list.focusUp"
	//        }
	//    ]"#;
	// 	assert_eq!(sort_array(input, "command", SortOrder::Asc), expected);
	// }
	// #[test]
	// fn sorts_keybindings_by_command_descending() {
	// 	let input = r#"[
	//        {
	//            "key": "up",
	//            "command": "list.focusUp"
	//        },
	//        {
	//            "key": "tab",
	//            "command": "editor.tab"
	//        },
	//        {
	//            "key": "ctrl+a",
	//            "command": "selectAll"
	//        }
	//    ]"#;
	// 	let expected = r#"[
	//        {
	//            "key": "up",
	//            "command": "list.focusUp"
	//        },
	//        {
	//            "key": "tab",
	//            "command": "editor.tab"
	//        },
	//        {
	//            "key": "ctrl+a",
	//            "command": "selectAll"
	//        }
	//    ]"#;
	// 	assert_eq!(sort_array(input, "command", SortOrder::Desc), expected);
	// }
	// #[test]
	fn can_sort_same_array_by_different_properties() {
		let input = r#"[
        {
            "key": "z",
            "command": "alpha"
        },
        {
            "key": "a",
            "command": "charlie"
        },
        {
            "key": "m",
            "command": "bravo"
        }
    ]"#;
		let by_key = r#"[
        {
            "key": "a",
            "command": "charlie"
        },
        {
            "key": "m",
            "command": "bravo"
        },
        {
            "key": "z",
            "command": "alpha"
        }
    ]"#;
		let by_command = r#"[
        {
            "key": "z",
            "command": "alpha"
        },
        {
            "key": "m",
            "command": "bravo"
        },
        {
            "key": "a",
            "command": "charlie"
        }
    ]"#;
		assert_eq!(sort_array(input, "key", SortOrder::Asc), by_key);
		assert_eq!(sort_array(input, "command", SortOrder::Asc), by_command);
	}
	#[test]
	fn missing_sort_property_sorts_last() {
		let input = r#"[
        {
            "key": "z",
            "command": "foo"
        },
        {
            "command": "bar"
        },
        {
            "key": "a",
            "command": "baz"
        }
    ]"#;
		let expected = r#"[
        {
            "key": "a",
            "command": "baz"
        },
        {
            "key": "z",
            "command": "foo"
        },
        {
            "command": "bar"
        }
    ]"#;
		assert_eq!(sort_array(input, "key", SortOrder::Asc), expected);
	}
	#[test]
	fn preserves_values_when_sorting() {
		let input = r#"{
            "z": "last",
            "a": "first",
            "m": "middle"
        }"#;
		let expected = r#"{
            "a": "first",
            "m": "middle",
            "z": "last"
        }"#;
		assert_eq!(sort_object(input, SortOrder::Asc), expected);
	}
	#[test]
	fn preserves_nested_objects() {
		let input = r#"{
            "z": {
                "nested": true,
                "values": [1, 2, 3]
            },
            "a": {
                "another": {
                    "deep": "value"
                }
            }
        }"#;
		let expected = r#"{
            "a": {
                "another": {
                    "deep": "value"
                }
            },
            "z": {
                "nested": true,
                "values": [1, 2, 3]
            }
        }"#;
		assert_eq!(sort_object(input, SortOrder::Asc), expected);
	}
	#[test]
	fn preserves_arrays_as_values() {
		let input = r#"{
            "z": [1, 2, 3],
            "a": ["foo", "bar"]
        }"#;
		let expected = r#"{
            "a": ["foo", "bar"],
            "z": [1, 2, 3]
        }"#;
		assert_eq!(sort_object(input, SortOrder::Asc), expected);
	}
	#[test]
	fn preserves_comments_attached_to_property() {
		let input = r#"{
            // Zed configuration
            "zed.setting": true,
            // Rust configuration
            "rust.setting": true,
            // General configuration
            "files.exclude": {}
        }"#;
		let expected = r#"{
            // General configuration
            "files.exclude": {},
            // Rust configuration
            "rust.setting": true,
            // Zed configuration
            "zed.setting": true
        }"#;
		assert_eq!(sort_object(input, SortOrder::Asc), expected);
	}
	#[test]
	fn preserves_property_formatting() {
		let input = r#"{
  "z": {
    "foo": true
  },
  "a": [
    1,
    2,
    3
  ]
}"#;
		let expected = r#"{
  "a": [
    1,
    2,
    3
  ],
  "z": {
    "foo": true
  }
}"#;
		assert_eq!(sort_object(input, SortOrder::Asc), expected);
	}
	#[test]
	fn does_not_sort_nested_objects() {
		let input = r#"{
            "z": {
                "b": 2,
                "a": 1
            },
            "a": {
                "d": 4,
                "c": 3
            }
        }"#;
		let expected = r#"{
            "a": {
                "d": 4,
                "c": 3
            },
            "z": {
                "b": 2,
                "a": 1
            }
        }"#;
		assert_eq!(sort_object(input, SortOrder::Asc), expected);
	}
	#[test]
	fn already_sorted_input_is_unchanged() {
		let input = r#"{
            "a": 1,
            "b": 2,
            "c": 3
        }"#;
		assert_eq!(sort_object(input, SortOrder::Asc), input);
	}
	#[test]
	fn descending_already_sorted_input_is_unchanged() {
		let input = r#"{
            "c": 3,
            "b": 2,
            "a": 1
        }"#;
		assert_eq!(sort_object(input, SortOrder::Desc), input);
	}
	#[test]
	fn handles_single_property() {
		let input = r#"{
            "only.key": true
        }"#;
		assert_eq!(sort_object(input, SortOrder::Asc), input);
	}
	#[test]
	fn handles_empty_object() {
		let input = r#"{}"#;
		assert_eq!(sort_object(input, SortOrder::Asc), input);
	}
	#[test]
	fn zed_keymap_contextless_unbind_items_come_first() {
		let input = r#"[
        {
            "bindings": {
                "cmd-y": "zed::Y",
                "cmd-b": "zed::B"
            },
            "unbind": {
                "cmd-z": "zed::Z",
                "cmd-a": "zed::A"
            }
        }
    ]"#;
		let expected = r#"[
        {
            "unbind": {
                "cmd-a": "zed::A",
                "cmd-z": "zed::Z"
            },
            "bindings": {
                "cmd-b": "zed::B",
                "cmd-y": "zed::Y"
            }
        }
    ]"#;
		assert_eq!(sort_zed_keymap(input, SortOrder::Asc), expected);
	}
	#[test]
	fn zed_keymap_contextless_binding_items_come_first() {
		let input = r#"[
        {
            "context": "Editor",
            "unbind": {
                "cmd-a": "editor::A"
            }
        },
        {
            "unbind": {
                "cmd-b": "editor::B"
            }
        },
        {
            "context": "Workspace",
            "unbind": {
                "cmd-c": "workspace::C"
            }
        }
    ]"#;
		let expected = r#"[
        {
            "unbind": {
                "cmd-b": "editor::B"
            }
        },
        {
            "context": "Editor",
            "unbind": {
                "cmd-a": "editor::A"
            }
        },
        {
            "context": "Workspace",
            "unbind": {
                "cmd-c": "workspace::C"
            }
        }
    ]"#;
		assert_eq!(sort_zed_keymap(input, SortOrder::Asc), expected);
	}
	#[test]
	fn zed_keymap_contextless_items_come_first() {
		let input = r#"[
        {
            "context": "Editor",
            "bindings": {
                "cmd-a": "editor::A"
            }
        },
        {
            "bindings": {
                "cmd-b": "editor::B"
            }
        },
        {
            "context": "Workspace",
            "bindings": {
                "cmd-c": "workspace::C"
            }
        }
    ]"#;
		let expected = r#"[
        {
            "bindings": {
                "cmd-b": "editor::B"
            }
        },
        {
            "context": "Editor",
            "bindings": {
                "cmd-a": "editor::A"
            }
        },
        {
            "context": "Workspace",
            "bindings": {
                "cmd-c": "workspace::C"
            }
        }
    ]"#;
		assert_eq!(sort_zed_keymap(input, SortOrder::Asc), expected);
	}
	#[test]
	fn zed_keymap_sorts_bindings_and_unbinds_independently() {
		let input = r#"[
        {
            "unbind": {
                "cmd-z": "zed::Z",
                "cmd-a": "zed::A"
            },
            "bindings": {
                "cmd-y": "zed::Y",
                "cmd-b": "zed::B"
            }
        }
    ]"#;
		let expected = r#"[
        {
            "unbind": {
                "cmd-a": "zed::A",
                "cmd-z": "zed::Z"
            },
            "bindings": {
                "cmd-b": "zed::B",
                "cmd-y": "zed::Y"
            }
        }
    ]"#;
		assert_eq!(sort_zed_keymap(input, SortOrder::Asc), expected);
	}
	#[test]
	fn zed_keymap_contexts_sort_alphabetically() {
		let input = r#"[
        {
            "context": "Workspace"
        },
        {
            "context": "Editor"
        },
        {
            "context": "Terminal"
        }
    ]"#;
		let expected = r#"[
        {
            "context": "Editor"
        },
        {
            "context": "Terminal"
        },
        {
            "context": "Workspace"
        }
    ]"#;
		assert_eq!(sort_zed_keymap(input, SortOrder::Asc), expected);
	}
	#[test]
	fn zed_keymap_contexts_sort_descending() {
		let input = r#"[
        {
            "context": "Editor"
        },
        {
            "context": "Workspace"
        },
        {
            "context": "Terminal"
        }
    ]"#;
		let expected = r#"[
        {
            "context": "Workspace"
        },
        {
            "context": "Terminal"
        },
        {
            "context": "Editor"
        }
    ]"#;
		assert_eq!(sort_zed_keymap(input, SortOrder::Desc), expected);
	}
	#[test]
	fn zed_keymap_bindings_sort_ascending() {
		let input = r#"[
        {
            "bindings": {
                "cmd-z": "zed::Z",
                "cmd-a": "zed::A",
                "cmd-m": "zed::M"
            }
        }
    ]"#;
		let expected = r#"[
        {
            "bindings": {
                "cmd-a": "zed::A",
                "cmd-m": "zed::M",
                "cmd-z": "zed::Z"
            }
        }
    ]"#;
		assert_eq!(sort_zed_keymap(input, SortOrder::Asc), expected);
	}
	#[test]
	fn zed_keymap_bindings_sort_descending() {
		let input = r#"[
        {
            "bindings": {
                "cmd-a": "zed::A",
                "cmd-z": "zed::Z",
                "cmd-m": "zed::M"
            }
        }
    ]"#;
		let expected = r#"[
        {
            "bindings": {
                "cmd-z": "zed::Z",
                "cmd-m": "zed::M",
                "cmd-a": "zed::A"
            }
        }
    ]"#;
		assert_eq!(sort_zed_keymap(input, SortOrder::Desc), expected);
	}
	#[test]
	fn zed_keymap_unbinds_sort_ascending() {
		let input = r#"[
        {
            "unbind": {
                "cmd-z": "zed::Z",
                "cmd-a": "zed::A",
                "cmd-m": "zed::M"
            }
        }
    ]"#;
		let expected = r#"[
        {
            "unbind": {
                "cmd-a": "zed::A",
                "cmd-m": "zed::M",
                "cmd-z": "zed::Z"
            }
        }
    ]"#;
		assert_eq!(sort_zed_keymap(input, SortOrder::Asc), expected);
	}
	#[test]
	fn zed_settings_sort_objects_recursively_ascending() {
		let input = r#"{
        "format_on_save": "on",
        "use_on_type_format": true,
        "autosave": "on_focus_change",
        "languages": {
            "Rust": {
                "format_on_save": "on"
            },
            "TypeScript": {
                "format_on_save": "on"
            }
        }
    }"#;
		let expected = r#"{
        "autosave": "on_focus_change",
        "format_on_save": "on",
        "languages": {
            "Rust": {
                "format_on_save": "on"
            },
            "TypeScript": {
                "format_on_save": "on"
            }
        },
        "use_on_type_format": true
    }"#;
		assert_eq!(sort_recursive(input, SortOrder::Asc), expected);
	}
	#[test]
	fn zed_settings_sort_objects_recursively_descending() {
		let input = r#"{
        "format_on_save": "on",
        "use_on_type_format": true,
        "autosave": "on_focus_change",
        "languages": {
            "Rust": {
                "format_on_save": "on"
            },
            "TypeScript": {
                "format_on_save": "on"
            }
        }
    }"#;
		let expected = r#"{
        "use_on_type_format": true,
        "languages": {
            "TypeScript": {
                "format_on_save": "on"
            },
            "Rust": {
                "format_on_save": "on"
            }
        },
        "format_on_save": "on",
        "autosave": "on_focus_change"
    }"#;
		assert_eq!(sort_recursive(input, SortOrder::Desc), expected);
	}
	#[test]
	fn zed_settings_sorts_deeply_nested_objects() {
		let input = r#"{
        "z": {
            "z": {
                "c": 3,
                "a": 1,
                "b": 2
            },
            "a": {
                "d": 4,
                "c": 3,
                "b": 2,
                "a": 1
            }
        },
        "a": {
            "z": 26,
            "b": 2,
            "m": 13
        }
    }"#;
		let expected = r#"{
        "a": {
            "b": 2,
            "m": 13,
            "z": 26
        },
        "z": {
            "a": {
                "a": 1,
                "b": 2,
                "c": 3,
                "d": 4
            },
            "z": {
                "a": 1,
                "b": 2,
                "c": 3
            }
        }
    }"#;
		assert_eq!(sort_recursive(input, SortOrder::Asc), expected);
	}
	#[test]
	fn zed_settings_does_not_sort_array_elements() {
		let input = r#"{
        "items": [
            {
                "z": 1,
                "a": 2
            },
            {
                "y": 3,
                "b": 4
            }
        ],
        "other": true
    }"#;
		let expected = r#"{
        "items": [
            {
                "a": 2,
                "z": 1
            },
            {
                "b": 4,
                "y": 3
            }
        ],
        "other": true
    }"#;
		assert_eq!(sort_recursive(input, SortOrder::Asc), expected);
	}
}
fn move_object_key_first(source: &str, key: &str) -> String {
	let root = CstRootNode::parse(source, &ParseOptions::default()).expect("invalid JSONC");
	let object = match root.object_value() {
		Some(object) => object,
		None => return source.to_string(),
	};
	let properties = object.properties();
	if properties.len() <= 1 {
		return source.to_string();
	}
	let target_index = match properties.iter().position(|property| {
		property
			.name()
			.and_then(|name| name.decoded_value().ok())
			.as_deref()
			== Some(key)
	}) {
		Some(index) => index,
		None => return source.to_string(),
	};
	if target_index == 0 {
		return source.to_string();
	}
	let mut entries = Vec::with_capacity(properties.len());
	for property in properties.iter() {
		let text = property.to_string();
		let start = source.find(&text).expect("property must exist");
		entries.push((start, text));
	}
	let first_start = entries[0].0;
	let last = entries.last().unwrap();
	let last_end = last.0 + last.1.len();
	let prefix = &source[..first_start];
	let suffix = &source[last_end..];
	// Keep the original separators/formatting.
	let mut chunks = Vec::with_capacity(entries.len());
	for (index, (start, text)) in entries.iter().enumerate() {
		let end = start + text.len();
		let next_start = entries
			.get(index + 1)
			.map(|(start, _)| *start)
			.unwrap_or(last_end);
		let between = &source[end..next_start];
		let (trailing, leading) = match between.find(',') {
			Some(comma) => (
				between[..=comma].to_string(),
				between[comma + 1..].to_string(),
			),
			None => (String::new(), between.to_string()),
		};
		chunks.push(SourceEntry {
			sort_key: None,
			leading,
			value: text.clone(),
			trailing,
		});
	}
	let target = chunks.remove(target_index);
	chunks.insert(0, target);
	let mut output = String::new();
	output.push_str(prefix);
	for entry in &chunks {
		output.push_str(&entry.value);
		output.push_str(&entry.trailing);
		output.push_str(&entry.leading);
	}
	output.push_str(suffix);
	output
}
