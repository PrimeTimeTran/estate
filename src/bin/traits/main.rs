fn main() {
	// # Introduction to Traits in Rust
	//
	// Traits are Rust's way of describing behavior independently
	// from the concrete types that implement that behavior.

	// 1. Capability
	// "What can this type do?"
	one::capability();

	// 2. Constraints
	// "What must this type be able to do?"
	two::constraints();

	// 3. Generic Abstraction
	// "Can I preserve the concrete type while abstracting over behavior?"
	three::generic_abstraction();

	// 4. Dispatch
	// "When and how does Rust select the implementation?"
	four::dispatch();

	// 5. Associated Types
	// "What types belong to this behavior?"
	five::associated_types();

	// 6. Composition
	// "How do behaviors depend on other behaviors?"
	six::composition();

	// 7. Dynamic Abstraction
	// "Can I erase the concrete type?"
	seven::dynamic_abstraction();

	// 8. Resolution
	// "Which implementation or method does Rust mean?"
	eight::resolution();

	// 9. Coherence
	// "Who is allowed to implement what?"
	nine::coherence();

	// 10. Advanced Trait Systems
	// "How far can Rust's trait system express relationships?"
	ten::advanced_trait_systems();
}

#[path = "sections/mod.rs"]
pub mod traits;
use traits::*;
