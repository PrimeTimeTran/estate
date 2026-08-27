# Rust Ownership

> **Core idea:** Rust's ownership system answers one question: **who is responsible for a value, and for how long?**

Ownership is not primarily about syntax. It is a set of rules governing **lifetime, mutation, borrowing, movement, and destruction** of values.

---

## 1. The Mental Model

Every value in Rust has:

- **An owner**
- **A lifetime**
- **A location in memory**
- **A way to access it**
- **A point at which it is destroyed**

The fundamental relationship is:

```text
Ownership
   │
   ├── Move
   ├── Borrow
   │    ├── Shared (&T)
   │    └── Mutable (&mut T)
   │
   ├── Clone
   └── Drop
```

Rust enforces these relationships **at compile time**.

The three foundational ownership rules are:

1. **Every value has an owner.**
2. **There can only be one owner at a time.**
3. **When the owner goes out of scope, the value is dropped.**

Borrowing adds the access rules:

```text
&T      → many readers
&mut T  → one writer
```

These cannot overlap in a way that permits data races.

---

## 2. Values, Bindings, and Owners

Consider:

```rust
let name = String::from("Alice");
```

There are two useful concepts here:

```text
name ──────owns──────> String("Alice")
```

`name` is a **binding**.

The `String` is the **value**.

The binding owns the value.

Ownership therefore belongs to a **value**, while variables/bindings are one mechanism through which ownership is held.

---

## 3. Stack vs Heap

Ownership becomes especially important when values contain heap allocation.

### Stack value

```rust
let x = 42;
```

Conceptually:

```text
stack
┌──────────┐
│ x = 42   │
└──────────┘
```

### Heap-owning value

```rust
let name = String::from("Alice");
```

Conceptually:

```text
stack                    heap
┌──────────────┐         ┌─────────┐
│ name         │────────>│ Alice   │
│ ptr          │         └─────────┘
│ len          │
│ capacity     │
└──────────────┘
```

The `String` value itself lives on the stack, but it **owns an allocation on the heap**.

Ownership is what makes Rust know who is responsible for that allocation.

---

## 4. Move

A **move transfers ownership**.

```rust
let a = String::from("hello");
let b = a;
```

After this:

```text
a ──X

b ──────owns──────> "hello"
```

`a` is no longer usable.

```rust
println!("{b}"); // okay
println!("{a}"); // error
```

The important thing is that Rust does not need to copy the heap allocation.

It can simply transfer ownership of the `String`'s representation.

```text
Before:

a ──────> allocation


After:

b ──────> allocation
```

This is why moves are cheap for many heap-owning types.

---

## 5. Copy

Some values implement `Copy`.

```rust
let a = 42;
let b = a;

println!("{a}");
println!("{b}");
```

Both remain valid.

The distinction is:

```text
Move:
ownership transfers

Copy:
value is duplicated implicitly
```

Primitive types such as integers, booleans, and characters are commonly `Copy`.

A type can implement `Copy` only when its contents can safely be duplicated by a simple bitwise copy.

---

## 6. Clone

`clone()` explicitly creates another value.

```rust
let a = String::from("hello");
let b = a.clone();
```

Now:

```text
a ──────> allocation A
b ──────> allocation B
```

Unlike a move, cloning can require an actual allocation and data copy.

Therefore:

```text
Copy  → implicit duplication
Clone → explicit duplication
Move  → ownership transfer
```

These are fundamentally different operations.

---

## 7. Drop

When an owner goes out of scope, Rust automatically calls `Drop`.

```rust
{
    let name = String::from("Alice");
} // name is dropped here
```

Conceptually:

```text
create
  ↓
own
  ↓
use
  ↓
scope ends
  ↓
drop
```

This is the foundation of **RAII** in Rust.

Resources such as:

- heap allocations
- files
- sockets
- locks
- database connections
- OS handles

can be tied to object lifetime.

---

## 8. Scope

Ownership is closely related to scope.

```rust
fn example() {
    let a = String::from("hello");

    {
        let b = String::from("world");
    } // b dropped

} // a dropped
```

The lifetime of a local binding is generally bounded by its scope.

But **scope and lifetime are not identical concepts**.

A Rust lifetime is primarily about **how long a reference is valid**, whereas a scope describes where a binding is accessible.

---

## 9. Borrowing

Borrowing allows access to a value **without taking ownership**.

```rust
let name = String::from("Alice");

let reference = &name;
```

Ownership remains:

```text
name ──────owns──────> String
reference ──borrows──> String
```

The reference does not destroy the value.

When `reference` disappears, `name` still owns the `String`.

---

## 10. Shared Borrow

`&T` creates a shared reference.

```rust
let name = String::from("Alice");

let a = &name;
let b = &name;
let c = &name;
```

Multiple shared references are allowed.

```text
        ┌── &name
        ├── &name
String ─┼── &name
        └── owner
```

The rule is:

> **Many readers are allowed.**

---

## 11. Mutable Borrow

`&mut T` creates a mutable reference.

```rust
let mut name = String::from("Alice");

let reference = &mut name;
reference.push_str(" Smith");
```

A mutable borrow provides exclusive access.

```text
        ┌── &mut T
String ─┤
        └── owner
```

The rule is:

> **One writer is allowed.**

While a mutable reference is active, incompatible accesses are prohibited.

---

## 12. The Alias XOR Mutability Rule

The core borrowing rule can be expressed as:

```text
many &T
    XOR
one &mut T
```

Not simultaneously.

That means:

```text
&value
&value
&value
```

is valid.

And:

```text
&mut value
```

is valid.

But:

```text
&value
&mut value
```

cannot overlap.

This prevents data races and iterator invalidation-style bugs at compile time.

---

## 13. Borrowing Does Not Transfer Ownership

Compare:

```rust
fn consume(value: String) {}
```

with:

```rust
fn inspect(value: &String) {}
```

Calling:

```rust
let name = String::from("Alice");

consume(name);
// name moved
```

versus:

```rust
let name = String::from("Alice");

inspect(&name);
// name still owned here
```

The distinction is fundamental:

```text
T       → ownership
&T      → shared access
&mut T  → exclusive access
```

---

## 14. Function Parameters

Function parameters are one of the clearest ways to understand ownership.

### Take ownership

```rust
fn consume(value: String) {
    // owns value
}
```

Calling:

```rust
consume(name);
```

moves `name`.

### Borrow immutably

```rust
fn inspect(value: &String) {
    // temporarily borrows value
}
```

Calling:

```rust
inspect(&name);
```

does not transfer ownership.

### Borrow mutably

```rust
fn modify(value: &mut String) {
    value.push_str("!");
}
```

Calling:

```rust
modify(&mut name);
```

temporarily grants exclusive access.

---

## 15. Return Values Transfer Ownership

Ownership can move through function returns.

```rust
fn create() -> String {
    String::from("hello")
}

let value = create();
```

Conceptually:

```text
create()
   │
   └── creates value
          │
          ▼
       caller owns it
```

Likewise:

```rust
fn transform(value: String) -> String {
    value
}
```

Ownership enters the function and leaves the function.

---

## 16. `.into()`

`into()` commonly represents an ownership-consuming conversion.

```rust
let a = String::from("hello");
let b: Vec<u8> = a.into();
```

The important part is the receiver:

```rust
a.into()
```

`into()` takes `self`.

Conceptually:

```text
a
│
└── into()
      │
      └── ownership consumed
              ↓
          new value
```

This is different from borrowing methods such as:

```rust
as_ref()
as_deref()
```

---

## 17. `.take()`

`take()` replaces a value with its default value and returns the original.

Most commonly:

```rust
let mut value = Some(String::from("hello"));

let old = value.take();
```

Afterward:

```text
value → None
old   → Some("hello")
```

This is useful when ownership must be extracted from a field without moving the entire containing object.

Conceptually:

```text
field
  │
  ├── old value → caller
  │
  └── replacement → Default
```

---

## 18. `mem::take()`

`std::mem::take()` is the generalized form of the same idea.

```rust
let mut value = vec![1, 2, 3];

let old = std::mem::take(&mut value);
```

Afterward:

```text
old   → [1, 2, 3]
value → []
```

It requires:

```rust
T: Default
```

The operation is essentially:

```text
replace value with T::default()
return old value
```

---

## 19. `mem::replace()`

`replace()` lets you choose the replacement value.

```rust
let mut value = String::from("old");

let old = std::mem::replace(
    &mut value,
    String::from("new"),
);
```

Result:

```text
old     → "old"
value   → "new"
```

The important ownership pattern is:

```text
&mut value
     │
     ├── extract old ownership
     └── install new ownership
```

This is particularly useful when working with struct fields.

---

## 20. `.unwrap()`

`unwrap()` consumes an `Option<T>` or `Result<T, E>`.

```rust
let value = Some(String::from("hello"));

let string = value.unwrap();
```

The `Option` is consumed and the inner `String` is returned.

Conceptually:

```text
Option<String>
      │
   unwrap()
      │
      ▼
   String
```

`unwrap()` is therefore an **ownership-consuming operation**.

---

## 21. `.drain()`

`drain()` creates an iterator that removes elements from a collection.

```rust
let mut values = vec![1, 2, 3];

for value in values.drain(..) {
    println!("{value}");
}
```

The elements are moved out of the collection.

Conceptually:

```text
Vec
│
├── element → iterator
├── element → iterator
└── element → iterator

Vec becomes empty
```

This is different from:

```rust
iter()
```

which only borrows the elements.

---

## 22. `iter()`

`iter()` produces shared references.

```rust
for value in values.iter() {
    // value: &T
}
```

Conceptually:

```text
Vec<T>
  │
  ├── &T
  ├── &T
  └── &T
```

Nothing is moved out.

---

## 23. `iter_mut()`

`iter_mut()` produces mutable references.

```rust
for value in values.iter_mut() {
    *value += 1;
}
```

Conceptually:

```text
Vec<T>
  │
  ├── &mut T
  ├── &mut T
  └── &mut T
```

The collection retains ownership.

The iterator temporarily provides mutable access to its elements.

---

## 24. `into_iter()`

`into_iter()` consumes the collection and yields owned values.

```rust
let values = vec![1, 2, 3];

for value in values.into_iter() {
    // value: i32
}
```

Conceptually:

```text
iter()       → &T
iter_mut()   → &mut T
into_iter()  → T
```

This distinction is one of the most useful ownership patterns in Rust.

---

## 25. `as_ref()`

`as_ref()` generally converts ownership-oriented containers into containers of references.

For example:

```rust
let value = Some(String::from("hello"));

let reference = value.as_ref();
```

Now:

```text
value     → owns String
reference → borrows String
```

The original container remains usable.

---

## 26. `as_mut()`

`as_mut()` is the mutable equivalent.

```rust
let mut value = Some(String::from("hello"));

if let Some(value) = value.as_mut() {
    value.push('!');
}
```

The `Option` remains the owner.

The returned value is a mutable reference into it.

---

## 27. `as_deref()`

`as_deref()` converts through `Deref`.

For example:

```rust
let value = Some(String::from("hello"));

let reference: Option<&str> = value.as_deref();
```

Conceptually:

```text
Option<String>
      │
   as_deref()
      ▼
Option<&str>
```

This is particularly useful when APIs expect borrowed string slices instead of owned `String`s.

---

## 28. `.borrow()`

`borrow()` is associated with the `Borrow` trait and, importantly, is **not the same thing as creating a normal Rust reference with `&`**.

For example:

```rust
use std::borrow::Borrow;

let value = String::from("hello");

let reference: &str = value.borrow();
```

It expresses a conversion from an owned representation into a borrowed representation.

The broader concept is:

```text
owned representation
        │
      borrow
        ▼
borrowed representation
```

---

## 29. `.deref()`

`deref()` comes from the `Deref` trait.

```rust
use std::ops::Deref;

let value = Box::new(String::from("hello"));

let reference: &String = value.deref();
```

It exposes the target behind a dereferenceable type.

Usually, Rust's **deref coercion** makes explicit `.deref()` unnecessary:

```rust
fn inspect(value: &str) {}

let value = String::from("hello");

inspect(&value);
```

Rust can automatically convert:

```text
&String
  ↓ deref coercion
&str
```

---

## 30. `.clone()` vs `.to_owned()`

These are related but conceptually different.

```rust
let a = String::from("hello");

let b = a.clone();
let c = a.to_owned();
```

`clone()` means:

> Create another instance of this value.

`to_owned()` means:

> Create an owned version of this borrowed value.

For example:

```rust
let slice: &str = "hello";

let owned: String = slice.to_owned();
```

A useful mental distinction:

```text
Clone
  → duplicate this value

ToOwned
  → create the owned form of this borrowed form
```

---

## 31. `.to_string()`

`to_string()` converts a displayable value into a `String`.

```rust
let value = 42;

let text = value.to_string();
```

For strings:

```rust
let slice: &str = "hello";
let owned = slice.to_string();
```

It creates owned data.

Thus:

```text
&str
 │
 └── to_string()
          ↓
       String
```

---

## 32. `.to_vec()`

`to_vec()` creates an owned `Vec` from a slice.

```rust
let values: &[i32] = &[1, 2, 3];

let owned = values.to_vec();
```

Conceptually:

```text
&[T]
 │
 └── to_vec()
       ↓
     Vec<T>
```

The resulting vector owns its elements.

---

## 33. `Arc`

`Arc<T>` provides shared ownership across threads.

```rust
use std::sync::Arc;

let value = Arc::new(String::from("hello"));

let a = Arc::clone(&value);
let b = Arc::clone(&value);
```

Conceptually:

```text
             ┌── Arc
             │
allocation ──┼── Arc
             │
             └── Arc
```

The allocation remains alive until the final `Arc` owner disappears.

---

## 34. `Arc::clone()`

`Arc::clone()` does **not** deep-copy the underlying value.

```rust
let a = Arc::new(data);
let b = Arc::clone(&a);
```

Both point to the same allocation.

```text
a ──┐
    ├──> T
b ──┘
```

Only the reference count is incremented.

This is fundamentally different from:

```rust
data.clone()
```

which may duplicate the actual data.

---

## 35. `Arc::try_unwrap()`

`try_unwrap()` attempts to recover ownership of the underlying value.

```rust
let value = Arc::new(String::from("hello"));

let owned = Arc::try_unwrap(value);
```

This succeeds only when there is a single strong owner.

Conceptually:

```text
Arc<T>
 │
 ├── one owner → T
 │
 └── many owners → cannot unwrap
```

This expresses:

> "I want ownership back, but only if nobody else owns this value."

---

## 36. `Box`

`Box<T>` provides unique ownership of a heap allocation.

```rust
let value = Box::new(String::from("hello"));
```

Conceptually:

```text
Box
 │
 └────owns────> heap allocation
```

Unlike `Arc`, a `Box` has a single owner.

---

## 37. `Box::into_inner`

`Box::into_inner` consumes the `Box` and returns the owned value.

Conceptually:

```text
Box<T>
  │
  └── into_inner()
          ↓
          T
```

Ownership moves from the heap allocation into the returned value.

---

## 38. `Box::leak`

`Box::leak()` intentionally turns an owned heap allocation into a reference that can live for an arbitrarily long lifetime, commonly `'static`.

```rust
let value = Box::new(String::from("hello"));

let reference: &'static String = Box::leak(value);
```

Conceptually:

```text
Box<T>
  │
  └── leak()
        ↓
      &'static T
```

The allocation is no longer automatically reclaimed through the original `Box`.

This is therefore an **intentional ownership escape**.

---

## 39. Lifetimes

A lifetime describes the period during which a reference is valid.

```rust
fn inspect<'a>(value: &'a String) -> &'a String {
    value
}
```

`'a` connects the input reference and output reference.

It says:

```text
the returned reference cannot outlive
the reference it came from
```

Lifetimes do not normally represent how long the owned value itself exists.

They primarily describe **relationships between references**.

---

## 40. `'static`

`'static` means a reference can remain valid for the entire program lifetime.

```rust
let value: &'static str = "hello";
```

String literals are `'static` because they are embedded in the program's binary.

But:

```text
'static
```

does **not** simply mean:

> "This value is permanent."

It means:

> "This reference is valid for the entire program lifetime."

An owned value can also be moved into a `'static` context without itself being a `'static` reference.

---

## 41. Lifetime Elision

Rust often infers lifetimes automatically.

Instead of:

```rust
fn inspect<'a>(value: &'a str) -> &'a str {
    value
}
```

you can write:

```rust
fn inspect(value: &str) -> &str {
    value
}
```

The compiler applies lifetime-elision rules.

Explicit lifetime annotations are primarily necessary when Rust needs help understanding **relationships between references**.

---

## 42. Ownership vs Borrowing

The cleanest distinction is:

```text
OWNERSHIP
    "I am responsible for this value."

BORROWING
    "I temporarily have access to this value."

MOVE
    "I transfer responsibility."

CLONE
    "I create another value."

DROP
    "I finish responsibility."

REFERENCE
    "I have access without responsibility."
```

---

## 43. Consuming vs Borrowing Methods

A useful way to inspect an unfamiliar API is to ask:

> **Does this method take `self`, `&self`, or `&mut self`?**

### `self`

Consumes ownership.

```rust
fn consume(self)
```

Think:

```text
ownership → method
```

Examples:

```text
into()
unwrap()
into_iter()
```

### `&self`

Borrows immutably.

```rust
fn inspect(&self)
```

Think:

```text
ownership
    ↓
temporary read access
```

Examples:

```text
len()
iter()
as_ref()
```

### `&mut self`

Borrows mutably.

```rust
fn modify(&mut self)
```

Think:

```text
ownership
    ↓
temporary exclusive access
```

Examples:

```text
iter_mut()
as_mut()
```

This is one of the most powerful ways to understand Rust APIs.

---

## 44. A Method Receiver Cheat Sheet

```text
self
│
└── consumes ownership

&self
│
└── shared borrow

&mut self
│
└── mutable borrow
```

And therefore:

```text
foo.into()       → likely consumes foo
foo.as_ref()     → likely borrows foo
foo.as_mut()     → likely mutably borrows foo
foo.iter()       → borrows elements
foo.iter_mut()   → mutably borrows elements
foo.into_iter()  → consumes foo
```

---

## 45. Ownership Extraction

Many Rust APIs exist because you cannot simply move a field out of a borrowed structure.

Consider:

```rust
struct State {
    value: String,
}
```

Given:

```rust
fn take_value(state: &mut State) -> String {
    // cannot simply move state.value out
}
```

You can instead replace it:

```rust
fn take_value(state: &mut State) -> String {
    std::mem::take(&mut state.value)
}
```

The pattern is:

```text
borrow structure mutably
        ↓
temporarily own field
        ↓
leave valid replacement behind
```

This pattern appears constantly in real Rust code.

---

## 46. Ownership and Collections

Collections make ownership particularly visible.

```rust
let mut values = vec![
    String::from("a"),
    String::from("b"),
];
```

The vector owns its elements:

```text
Vec
 │
 ├── owns String
 └── owns String
```

Then:

```rust
values.iter()
```

gives:

```text
&String
```

while:

```rust
values.iter_mut()
```

gives:

```text
&mut String
```

and:

```rust
values.into_iter()
```

gives:

```text
String
```

The entire distinction is:

```text
           access to elements

iter()      → borrow
iter_mut()  → mutable borrow
into_iter() → ownership
```

---

## 47. Dereferencing

If:

```rust
let value = 42;
let reference = &value;
```

then:

```rust
*reference
```

accesses the value behind the reference.

Conceptually:

```text
reference
    │
    ▼
  value
```

`&` creates a reference.

`*` dereferences one.

```text
&T
 │
 └── * → T
```

In expressions, Rust automatically inserts some borrowing and dereferencing through coercion and method lookup.

---

## 48. Ownership Through Smart Pointers

Rust's pointer types encode different ownership models.

```text
Box<T>
  → one owner
  → heap allocation

Rc<T>
  → shared ownership
  → single-threaded

Arc<T>
  → shared ownership
  → thread-safe reference counting

& T
  → borrowed access

&mut T
  → exclusive borrowed access
```

These are not merely different pointer implementations.

They represent different **ownership semantics**.

---

## 49. `Rc::clone()` vs `Arc::clone()`

Both create another owner without cloning the underlying `T`.

```rust
let a = Rc::new(value);
let b = Rc::clone(&a);
```

and:

```rust
let a = Arc::new(value);
let b = Arc::clone(&a);
```

Conceptually:

```text
Rc<T> / Arc<T>
      │
      ├── owner
      ├── owner
      └── shared T
```

The difference is primarily the environment in which ownership is safe:

```text
Rc  → single-threaded
Arc → thread-safe sharing
```

---

## 50. Ownership Conversion Vocabulary

A useful vocabulary map:

```text
MOVE
  into()
  unwrap()
  into_iter()
  drain()
  Box::into_inner()
  Arc::try_unwrap()

COPY
  implicit for Copy types

CLONE
  clone()
  Rc::clone()
  Arc::clone()

BORROW
  &value
  iter()
  as_ref()
  borrow()
  deref()

MUTABLY BORROW
  &mut value
  iter_mut()
  as_mut()

CREATE OWNED VALUE
  to_owned()
  to_string()
  to_vec()

REPLACE OWNED VALUE
  mem::take()
  mem::replace()
  Option::take()

ESCAPE OWNERSHIP
  Box::leak()
```

---

## 51. The Fundamental Question

When reading Rust code, do not initially ask:

> "What does this syntax do?"

Ask:

> **"What happens to ownership here?"**

For every operation, classify it:

```text
Does it...

[ ] Move the value?
[ ] Copy the value?
[ ] Clone the value?
[ ] Borrow the value?
[ ] Mutably borrow the value?
[ ] Consume the owner?
[ ] Extract ownership from somewhere?
[ ] Create a new owner?
[ ] Replace an existing value?
[ ] Extend a reference's lifetime?
[ ] Drop a value?
```

Once this becomes automatic, much of Rust's syntax becomes predictable.

---

## 52. The Ownership Lifecycle

A value can be understood as moving through this lifecycle:

```text
                 CREATE
                    │
                    ▼
                OWNERSHIP
                    │
          ┌─────────┼─────────┐
          │         │         │
        BORROW     MOVE      CLONE
          │         │         │
          │         ▼         ▼
          │      NEW OWNER  NEW VALUE
          │
          ├── &T
          │
          └── &mut T
                    │
                    ▼
                  DROP
```

The compiler's job is largely to ensure that these transitions are valid.

---

## 53. The Three Questions

When confused by Rust ownership, reduce the problem to three questions:

### 1. Who owns this value?

```text
variable?
struct field?
Box?
Rc?
Arc?
```

### 2. Who currently has access?

```text
owner?
&T?
&mut T?
```

### 3. When does that access end?

```text
scope?
last use?
explicit lifetime relationship?
```

If you can answer those three questions, most borrow-checker errors become understandable.

---

## 54. The Core Model

The entire ownership system can be compressed into this:

```text
                 VALUE
                   │
          ┌────────┴────────┐
          │                 │
       OWNED              BORROWED
          │                 │
     ┌────┴────┐       ┌────┴────┐
     │         │       │         │
   unique    shared    &T       &mut T
     │         │       │         │
    Box       Rc/Arc   many      one
     │
     ▼
   Drop
```

Or, more simply:

```text
              Ownership
                  │
       ┌──────────┼──────────┐
       │          │          │
      Move      Borrow     Clone
       │          │          │
       │      ┌───┴───┐      │
       │      │       │      │
       │     &T     &mut T   │
       │      │       │      │
       │    shared  exclusive│
       │                      │
       └──────────┬───────────┘
                  │
                 Drop
```

> **Rust's ownership system is fundamentally a system for controlling who may access a value, who is responsible for destroying it, and how those responsibilities move over time.**
