#![allow(warnings)]

fn main() {
	main_intro()
}
fn main_intro() {
	trait Trait {}
	trait Bar {}
	trait Foo {
		type GenericAssociatedType<'a>;
	}
	impl Foo for u32 {
		type GenericAssociatedType<'a> = &'a u32;
	}
	/// For every T that implements Foo, the particular type
	///
	/// (T, for<'a> fn(T::GenericAssociatedType<'a>))
	///
	/// implements Trait.
	///
	/// - You're implementing it for a constructed type involving T.
	///
	impl<T: Foo> Trait for (T, for<'a> fn(<T as Foo>::GenericAssociatedType<'a>)) {}
	// -> If I can prove T: Foo, then I can prove that this derived type implements Trait.

	// impls input type must satisfy `Trait`

	fn impls<T: Trait>() {}
	impls::<(u32, for<'a> fn(&'a u32))>();
}
fn _main_documented() {
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
	/// -> “Given a lifetime 'a, this associated type produces a type.”
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
	impl<T: OtherTrait> Trait for (T, for<'a> fn(<T as OtherTrait>::Assoc<'a>)) {}

	fn impls<T: Trait>() {}

	impls::<(u32, for<'a> fn(&'a u32))>();

	// Constructed Type
	// -> A type formed by applying type constructors to other types.
	//
	// (T, for<'a> fn(<T as Foo>::GenericAssociatedType<'a>))
}
