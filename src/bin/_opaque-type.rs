#![allow(warnings)]

/// ============================================================
/// EXAMPLE 1: recursive `impl Trait`
/// ============================================================
/// `impl Trait` in a return position creates an OPAQUE TYPE.
///
/// The function:
///
///     fn foo(b: bool) -> impl Sized
///
/// means:
///
///     "foo returns some concrete type T such that T: Sized,
///      but the caller is not allowed to name T."
///
/// The compiler may know the exact type internally, but it hides it from
/// outside code. This is why it is called "opaque": the type is real and
/// known to the compiler, but opaque to users of the function.
///
/// The important question is: what type is chosen for `foo`?
///
/// In this example:
///
///     fn foo(b: bool) -> impl Sized {
///         if b { foo(false) + 1 } else { 0 }
///     }
///
/// both return paths must produce the same hidden type.
///
/// - In the `else` branch, `0` is an integer literal.
/// - Integer literals are not a single fixed type; they are inferred to some
///   concrete integer type such as `i32` or `u32`.
/// - So the compiler ends up inferring that the hidden return type is some
///   integer-like concrete type.
///
/// Then the `if` branch does:
///
///     foo(false) + 1
///
/// That means the recursive call `foo(false)` must produce the same hidden
/// type as the outer call, and that type must support `+ 1`.
///
/// So the compiler is solving a constraint like:
///
///     hidden_type = same hidden type as recursive call
///     hidden_type = some integer type
///     hidden_type: Sized
///
/// That is the core trick: the type is hidden from callers, but not from the
/// type checker. The solver must prove that all return paths agree on one
/// concrete hidden type.
///
/// This is a good example of "recursive reasoning about an opaque type":
/// the function returns the same opaque type on recursive calls, and the
/// caller cannot see it, but the compiler still has to reason about it
/// sufficiently to verify that every branch is type-correct.
///
/// Without the newer solver, this kind of recursive constraint can be hard
/// to prove.
///
/// ------------------------------------------------------------------
/// Why this is not "infinite recursion" at the type level:
/// ------------------------------------------------------------------
/// The function is still a runtime recursive function, but it only recurses
/// until `b` becomes false. The type system is not trying to prove that the
/// function never recurses; it is trying to prove that all recursive call
/// sites agree on the same hidden return type.
/// ------------------------------------------------------------------

fn foo(b: bool) -> impl Sized {
	if b { foo(false) + 1 } else { 0 }
}

fn main() {
	let hi = foo(true);
}
