#![allow(warnings)]
#[path = "./mod.rs"]
mod lifetime;
use lifetime::*;

// Introduction to Lifetimes in Rust
//
// Explore data as it progresses through a normal lifecycle
// and the concepts introduced by Rust which help to build robust
// programs that don't have the same problems have earlier generations
// such as
//  - Memory
fn main() {
  // 1. Memory
  // The setting in which data & memory usage appears throughout any app
  // or programming language
	one::memory_considerations();

	// 2. Sharing
	two::sharing_forces_new_abstractions();

	// 3. Lifetimes
	three::three();

	// 4. Functions
	// Structs have unit fields attached automatically.
	four::four();
	// 5. Structs
	five::five();
	// 6. Ownership Transfer
	six::six();
	// 7. Transformation & Duplication
	seven::seven();

	// 8. Collections & Iteration
	// ## 9. Lifetimes in Types

	// ## 12. Advanced Lifetimes
	// - Higher-ranked trait bounds (`for<'a>`)
	// - Higher-order borrowing
	// - Lifetime subtyping
	// - Variance
	// - PhantomData
	// - Interior mutability
	// - Async lifetimes
	// - Pinning
	// - Self-referential types

	// ## 13. Mental Model
	// - Ownership answers: **Who owns this?**
	// - Borrowing answers: **Who can access this?**
	// - Lifetimes answer: **How long is that access valid?**
	// - Types answer: **What kind of thing is this?**
	// - `Drop` answers: **When is the owned resource destroyed?**

	// ## 14. Lifetime Review
	// - Identify the owner
	// - Identify every borrow
	// - Identify the borrow's required lifetime
	// - Determine which lifetime outlives which
	// - Check mutation/exclusivity
	// - Decide whether ownership should be transferred instead
	// - Decide whether the data should be cloned/copied
}

// Memory
//    ↓
// Ownership
//    ↓
// Borrowing
//    ↓
// Sharing
//    ↓
// References
//    ↓
// Lifetimes
//    ↓
// Lifetime Constraints
//    ↓
// Lifetime-bearing Types
//    ↓
// Abstractions that make those constraints manageable
// 
// # Introduction to Lifetimes in Rust
