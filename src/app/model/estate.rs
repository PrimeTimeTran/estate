//! Core domain types and workspace discovery.
//! # Description
//! This module contains:
//!
//! - [`Estate`]
//! - [`Node`]
//! - [`Resource`]
//! - [`Relation`]
//! - [`EstateDiscovery`]

use crate::prelude::*;

/// Represents an Estate and its complete project state.
///
/// An [`Estate`] is the root entity for a project. It owns the project's
/// identity, scope, nodes, resources, relations, and bindings.
///
/// Each Estate has a globally unique [`Uuid`] and may optionally have a
/// parent Estate, allowing Estates to be organized hierarchically.
///
/// # Resources
///
/// Resources represent files or other external assets associated with the
/// Estate. They can be created, looked up, mutably accessed, and removed
/// through the resource methods on this type.
///
/// # Examples
///
/// ```
/// let estate = Estate::new("my-project".into(), Scope::default());
/// assert_eq!(estate.resources.len(), 0);
/// ```
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Estate {
	pub bindings: Vec<Binding>,
	pub id: Uuid,
	pub name: String,
	pub nodes: Vec<Node>,
	pub parent: Option<Uuid>,
	pub relations: Vec<Relation>,
	pub resources: Vec<Resource>,
	pub scope: Scope,
}

/// Estate Constructors
///
/// Constructors of Estate Entities
impl Estate {
	/// Initializes an Estate Entity.
	///
	/// The Estate starts with no parent, nodes, resources, relations, or
	/// bindings.
	pub fn new(name: String, scope: Scope) -> Self {
		Self {
			bindings: Vec::new(),
			id: Uuid::new_v4(),
			name,
			nodes: Vec::new(),
			parent: None,
			relations: Vec::new(),
			resources: Vec::new(),
			scope,
		}
	}
}

impl Estate {
	/// Adds a resource to the Estate and returns its identifier.
	///
	/// The resource's existing [`Resource::id`] is preserved.
	pub fn create_resource(&mut self, resource: Resource) -> Uuid {
		let id = resource.id;
		self.resources.push(resource);
		id
	}

	/// Returns a reference to the resource with the given identifier.
	///
	/// Returns `None` if the Estate does not contain a matching resource.
	pub fn resource(&self, id: Uuid) -> Option<&Resource> {
		self.resources.iter().find(|r| r.id == id)
	}

	/// Returns a mutable reference to the resource with the given identifier.
	///
	/// Returns `None` if the Estate does not contain a matching resource.
	pub fn resource_mut(&mut self, id: Uuid) -> Option<&mut Resource> {
		self.resources.iter_mut().find(|r| r.id == id)
	}

	/// Removes the resource with the given identifier from the Estate.
	///
	/// Returns the removed resource if it existed, otherwise `None`.
	pub fn remove_resource(&mut self, id: Uuid) -> Option<Resource> {
		let index = self.resources.iter().position(|r| r.id == id)?;
		Some(self.resources.remove(index))
	}
}
