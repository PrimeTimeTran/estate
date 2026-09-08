// #![allow(warnings)]

fn main() {}

// use estate::prelude::*;

// // ============================================================
// // EXAMPLE 1: recursive `impl Trait`
// // ============================================================
// // Can the trait solver reason about the properties of this opaque type even
// // though determining those properties involves going back through the opaque type itself?
// fn foo(b: bool) -> impl Sized {
// 	if b { foo(false) + 1 } else { 0 }
// }

// // ============================================================
// // EXAMPLE 2: Generic Associated Types
// // ============================================================
// // "For any T that implements OtherTrait, the tuple
// // consisting of T and a function that accepts T's associated
// // type for any lifetime implements Trait."

// trait OtherTrait {
// 	type Assoc<'a>;
// }
// impl OtherTrait for u32 {
// 	type Assoc<'a> = &'a u32;
// }

// trait Trait {}
// impl<T: OtherTrait> Trait for (T, for<'a> fn(<T as OtherTrait>::Assoc<'a>)) {}

// fn impls<T: Trait>() {}

// fn main() {
// 	foo(true);
// 	// The old implementation failed to prove
// 	// the where-bound of `impls`.
// 	impls::<(u32, for<'a> fn(&'a u32))>();
// 	foo(false);
// 	()
// }

// // fn main1() {

// // }

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
