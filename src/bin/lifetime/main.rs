#![allow(warnings)]

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

// Introduction to Lifetimes in Rust
//
// Explore data as it progresses through a normal lifecycle
// and the concepts introduced by Rust which help to build robust
// programs that don't have the same problems earlier generations of languages had.

fn main() {
	// # Introduction to Lifetimes in Rust
	//
	// 1. Memory
	// The setting in which data & memory usage appears throughout any app or language
	one::memory_considerations();

	// 2. Sharing
	two::ownership_borrows_sharing();

	// 3. Lifetimes
	three::lifetimes();

	// 4. Functions
	// Structs have unit fields attached automatically.
	four::functions();

	// 5. Structs
	five::structs_with_lifetimes();

	// // 6. Ownership Transfer
	// six::six();

	// // 7. Transformation & Duplication
	// seven::seven();

	// // 8. Collections & Iteration
	// eight::eight();

	// // 9. Lifetimes in Types
	// nine::nine();

	// // 10. Lifetime Patterns
	// ten::ten();

	// // 11. Common Lifetime Problems
	// eleven::eleven();

	// // 12. Advanced Lifetimes
	// twelve::twelve();

	// // 13. Mental Model
	// thirteen::thirteen();

	// // 14. Lifetime Review
	// fourteen::fourteen();
}

use owo_colors::OwoColorize;

#[path = "./sections/mod.rs"]
mod lifetime;
use lifetime::*;

// #[path = "./sections/mod.rs"]
mod helpers;
use helpers::*;
