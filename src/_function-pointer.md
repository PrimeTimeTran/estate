# Function Pointer

## Function Item

A function item is the concrete, compiler-generated type of a
particular function definition.

```rust
fn add(a: u32, b: u32) -> u32 {
	a + b
}
```

The function item is the function itself, including its identity
and implementation. Its type is distinct from a function pointer type.

## Function Pointer Type

A function pointer is a value that points to a function.

```rust
fn(u32, u32) -> u32
```

Read as:
"A function pointer that accepts two u32s and returns a u32."

To use this type in an actual Rust declaration:

```rust
let f: fn(u32, u32) -> u32 = add;
```

Here:

- `fn(u32, u32) -> u32` is the function pointer type.
- `f` is a value of that type.
- `add` is coerced from a function item into a function pointer.

## Function Pointer Type with a Reference Argument

```rust
fn(&u32)
```

This is a function pointer type that accepts a reference to a u32.

In a declaration:

```rust
let f: fn(&u32) = |x| println!("{x}");
```

The lifetime of the reference is elided. Rust infers the appropriate
lifetime relationship for this function-pointer type.

## Higher-Ranked Function Pointer Type

```rust
for<'a> fn(&'a u32)
```

This is a higher-ranked function pointer type.

`for<'a>` is a lifetime binder, NOT a for loop.

Read as:
"A function pointer that can accept a reference to a u32 for ANY
lifetime 'a."

`for<'a>` universally quantifies over the lifetime:

    for every lifetime 'a

the function pointer can accept an `&'a u32`.

To use it in an actual Rust declaration:

```rust
let f: for<'a> fn(&'a u32) = |x| println!("{x}");
```

The important abstraction is:

    for<'a> fn(&'a u32)
    ^^^^^^^
    Higher-ranked lifetime binder

    fn(&'a u32)
    ^^^^^^^^^^^
    Function pointer type parameterized by that lifetime
