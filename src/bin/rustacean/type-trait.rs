#![allow(warnings)]

fn main() {
	/// ## Generic Associated Types (GATs)
	///
	/// A GAT is an associated type that is itself parameterized:
	///
	///     trait OtherTrait {
	///         type Assoc<'a>;
	///     }
	///
	/// Unlike a regular associated type, `Assoc<'a>` is a family of types:
	///
	///     'a -> type
	///
	/// For `u32`, the implementation defines that family as:
	///
	///     <u32 as OtherTrait>::Assoc<'a> = &'a u32
	///
	/// Now consider:
	///
	///     impl<T: OtherTrait> Trait
	///         for (T, for<'a> fn(<T as OtherTrait>::Assoc<'a>)) {}
	///
	/// This says that any `T: OtherTrait` satisfies `Trait` when paired with
	/// a function pointer that accepts `T`'s associated type for every lifetime.
	///
	/// The test:
	///
	///     impls::<(u32, for<'a> fn(&'a u32))>();
	///
	/// requires the solver to establish:
	///
	///     (u32, for<'a> fn(&'a u32)): Trait
	///
	/// To apply the impl, it must:
	///
	///     1. Match `T` with `u32`.
	///     2. Prove `u32: OtherTrait`.
	///     3. Normalize `<u32 as OtherTrait>::Assoc<'a>` to `&'a u32`.
	///     4. Verify that the resulting tuple matches the requested type.
	///
	/// The resulting substitution is:
	///
	///     (T, for<'a> fn(<T as OtherTrait>::Assoc<'a>))
	///         => (u32, for<'a> fn(&'a u32))
	///
	/// This tests GAT normalization, higher-ranked lifetime binders, and
	/// trait matching across an associated-type substitution.
	///
	/// Unlike the opaque-type example, this is not primarily testing recursive
	/// opaque-type solving. It tests whether the solver can normalize a
	/// lifetime-parameterized associated type and use that result to match
	/// an impl.
	// OtherTrait is a contract.
	// One requirement of that contract is that each implementor
	// supplies a type family called Assoc, parameterized by a lifetime.
	trait OtherTrait {
		/// An associated type that takes a lifetime parameter and produces a type.
		type Assoc<'a>;
	}
	impl OtherTrait for u32 {
		type Assoc<'a> = &'a u32;
	}

	trait Trait {}

	/// The trait `Trait` is being implemented on a tuple literal.
	/// - 0 index is the type.
	/// - 1 index is a 'higher-ranked lifetime binder'
	///
	/// [function-pointer type]: A pointer to a function that accepts
	/// a higher-ranked lifetime binder.
	///
	// or, for a real link:
	/// See the [function-pointer explanation](./function-pointer.md).
	///
	/// This blanket implementation applies to any `T: OtherTrait` where the
	/// tuple contains a function pointer accepting `T::Assoc<'a>` for every
	/// lifetime `'a`.
	trait OtherTrait {
		type Assoc<'a>;
	}
	impl OtherTrait for u32 {
		type Assoc<'a> = &'a u32;
	}
	trait Trait {}
	impl<T: OtherTrait> Trait for (T, for<'a> fn(<T as OtherTrait>::Assoc<'a>)) {}
	// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ [function-pointer type]

	fn impls<T: Trait>() {}
	impls::<(u32, for<'a> fn(&'a u32))>();
}

// // fn fooo(b: bool) -> impl Sized {
// // 	if b { foo(false) + 1 } else { 0 }
// // }
// // // ============================================================
// // // EXAMPLE 1: recursive `impl Trait`
// // // ============================================================
// // //
// // // `impl Sized` means:
// // //
// // //   "foo returns some concrete type that implements Sized,
// // //    but the caller doesn't get to know what that concrete
// // //    type is."
// // //
// // // IMPORTANT:
// // // `impl Trait` in a return position creates an OPAQUE type.
// // //
// // // Think:
// // //
// // //   fn foo(...) -> SomeHiddenType
// // //
// // // except the compiler hides `SomeHiddenType` from callers.
// // //
// // // The interesting question is:
// // //   What happens when that hidden type depends on itself?
// // //

// // fn foooo(b: bool) -> impl Sized {
// // 	if b {
// // 		// `foo(false)` itself returns the SAME opaque return type
// // 		// as `foo(true)`.
// // 		//
// // 		// Then we do:
// // 		//
// // 		//     foo(false) + 1
// // 		//
// // 		// So the compiler has to reason about the hidden type
// // 		// returned by `foo`.
// // 		//
// // 		// The old trait solver had trouble proving that the
// // 		// resulting expression was valid.
// // 		//
// // 		// In particular, it needs to reason recursively about
// // 		// the opaque type produced by `foo`.
// // 		//
// // 		// The new trait solver is better at this kind of
// // 		// recursive reasoning.

// // 		// The old implementation errored here.
// // 		foo(false) + 1
// // 	} else {
// // 		// Here the concrete value is simply an integer.
// // 		//
// // 		// Therefore, conceptually, the compiler can determine
// // 		// that the opaque return type is an integer type.
// // 		//
// // 		// Roughly:
// // 		//
// // 		//     foo(...) -> some hidden integer type
// // 		//
// // 		// and because `+ 1` is also valid for that type,
// // 		// the recursive branch can be checked.
// // 		//
// // 		// The exact compiler representation is more subtle,
// // 		// but thinking "hidden concrete integer type" is useful.

// // 		0
// // 	}
// // }

// // // ============================================================
// // // EXAMPLE 2: Generic Associated Types
// // // ============================================================

// // trait OtherTraitt {
// // 	type Assoc<'a>;
// // }

// // impl OtherTraitt for u32 {
// // 	type Assoc<'a> = &'a u32;
// // }
// // trait Traitt {}
// // impl<T: OtherTraitt> Traitt for (T, for<'a> fn(<T as OtherTraitt>::Assoc<'a>)) {}

// // fn implss<T: Traitt>() {}

// // fn main2() {
// // 	implss::<(u32, for<'a> fn(&'a u32))>();
// // }
// // // Now we're testing a completely different part of the trait
// // // solver.
// // //
// // // `OtherTrait` has an associated type named `Assoc`.
// // //
// // // The `'a` means `Assoc` is a GENERIC associated type (GAT).
// // //
// // // That is, the associated type isn't one fixed type.
// // //
// // // Instead, it is a type-producing function:
// // //
// // //     'a -> some type
// // //
// // // So you can conceptually think:
// // //
// // //     OtherTrait::Assoc<'a>
// // //
// // // as:
// // //
// // //     "give me the associated type for lifetime 'a"
// // //

// // trait OtherTraittt {
// // 	// For every lifetime `'a`, this trait specifies an
// // 	// associated type.
// // 	//
// // 	// The lifetime is an input to the associated type.

// // 	type Assoc<'a>;
// // }

// // // ------------------------------------------------------------
// // // Implement OtherTrait for u32
// // // ------------------------------------------------------------

// // impl OtherTraittt for u32 {
// // 	// For `u32`, we've decided that:
// // 	//
// // 	//     Assoc<'a> = &'a u32
// // 	//
// // 	// In other words:
// // 	//
// // 	//     <u32 as OtherTrait>::Assoc<'a>
// // 	//
// // 	// becomes:
// // 	//
// // 	//     &'a u32
// // 	//
// // 	// for whatever lifetime `'a` we're talking about.

// // 	type Assoc<'a> = &'a u32;
// // }

// // // ============================================================
// // // A second trait
// // // ============================================================

// // trait Traittt {}

// // // ------------------------------------------------------------
// // // Implement Trait for a PARTICULAR SHAPE of tuple
// // // ------------------------------------------------------------
// // //
// // // This is the really interesting part.
// // //
// // // We're saying:
// // //
// // //     If T implements OtherTrait,
// // //     then Trait is implemented for:
// // //
// // //         (T, for<'a> fn(<T as OtherTrait>::Assoc<'a>))
// // //
// // // Let's break that apart.
// // // ------------------------------------------------------------

// // impl<T: OtherTraitt> Traittt
// // 	for (
// // 		T,
// // 		// `for<'a>` means:
// // 		//
// // 		//     "for EVERY possible lifetime 'a"
// // 		//
// // 		// This is a higher-ranked trait bound (HRTB)-style
// // 		// function pointer type.
// // 		//
// // 		// So this is NOT one particular function pointer
// // 		// tied to one particular lifetime.
// // 		//
// // 		// It is a function pointer that can accept the
// // 		// associated type for ANY lifetime.
// // 		//
// // 		// Conceptually:
// // 		//
// // 		//     for every 'a:
// // 		//         fn(<T as OtherTrait>::Assoc<'a>)
// // 		//
// // 		// The important word is:
// // 		//
// // 		//     EVERY
// // 		//
// // 		for<'a> fn(<T as OtherTraittt>::Assoc<'a>),
// // 	)
// // {
// // }

// // // ============================================================
// // // A function that requires Trait
// // // ============================================================
// // //
// // // This function doesn't actually do anything.
// // //
// // // Its purpose is to force the compiler to answer:
// // //
// // //     "Does T implement Trait?"
// // //
// // // If the compiler can prove the bound, this compiles.

// // fn implsss<T: Traittt>() {}

// // // ============================================================
// // // Put everything together
// // // ============================================================

// // // fn main3() {
// // // 	// We ask:
// // // 	//
// // // 	//     Does this type implement Trait?
// // // 	//
// // // 	// Specifically:
// // // 	//
// // // 	//     (u32, for<'a> fn(&'a u32))
// // // 	//
// // // 	// We already know:
// // // 	//
// // // 	//     impl OtherTrait for u32
// // // 	//
// // // 	// and therefore:
// // // 	//
// // // 	//     <u32 as OtherTrait>::Assoc<'a>
// // // 	//
// // // 	// is:
// // // 	//
// // // 	//     &'a u32
// // // 	//
// // // 	// Now look back at the Trait implementation:
// // // 	//
// // // 	//     impl<T: OtherTrait> Trait
// // // 	//         for (
// // // 	//             T,
// // // 	//             for<'a> fn(<T as OtherTrait>::Assoc<'a>)
// // // 	//         )
// // // 	//
// // // 	// Substitute:
// // // 	//
// // // 	//     T = u32
// // // 	//
// // // 	// The implementation becomes approximately:
// // // 	//
// // // 	//     impl Trait for (
// // // 	//         u32,
// // // 	//         for<'a> fn(&'a u32)
// // // 	//     )
// // // 	//
// // // 	// Which is EXACTLY the type we're passing below.
// // // 	//
// // // 	// So mathematically / logically:
// // // 	//
// // // 	//     u32: OtherTrait
// // // 	//
// // // 	// therefore:
// // // 	//
// // // 	//     <u32 as OtherTrait>::Assoc<'a> = &'a u32
// // // 	//
// // // 	// therefore:
// // // 	//
// // // 	//     (u32, for<'a> fn(&'a u32)): Trait
// // // 	//
// // // 	//
// // // 	// The old trait solver had trouble making this chain of
// // // 	// deductions.
// // // 	//
// // // 	// The new solver can prove it.

// // // 	// The old implementation failed to prove
// // // 	// the where-bound of `impls`.
// // // 	implsss::<(u32, for<'a> fn(&'a u32))>();

// // // 	// # 1
// // // 	// "For any T that implements OtherTrait, the tuple
// // // 	// consisting of T and a function that accepts T's associated type for any lifetime implements Trait."

// // // 	impl<T: OtherTraittt> Traittt for (T, for<'a> fn(<T as OtherTraittt>::Assoc<'a>)) {}

// // // 	fn spams() {}
// // // }
