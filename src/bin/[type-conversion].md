# Type Conversion


- [Type inference](#1-type-inference--no-conversion-occurs)
- [Coercion](#2-coercion)
- [Conversion](#3-conversion)
- [Casting](#4-casting)

### 1. Type inference — no conversion occurs

```rust
let n = 10;
let x: i64 = n;
```

The subtle thing you were exploring here is that `10` is an **unsuffixed integer literal**. The compiler hasn't necessarily committed it to `i32` yet.

The constraint from:

```rust
let x: i64 = n;
```

causes the inferred program to effectively become:

```rust
let n: i64 = 10;
let x: i64 = n;
```

There is **no `i32 → i64` conversion** happening.

That's worth keeping separate from everything below.

---

# 2. `as` — explicit primitive casting

This is probably the next thing I'd learn after `From`/`Into`.

```rust
let n: i32 = 10;
let x: i64 = n as i64;
```

Now there really is a conversion/cast:

```text
i32
 │
 │ as
 ▼
i64
```

And unlike your previous example, `n` really is an `i32`.

You can also do things like:

```rust
let x: u8 = 300_i32 as u8;
```

which gives you truncation/wrapping behavior rather than a compile-time error.

So mentally:

```rust
as
```

is the **explicit primitive casting mechanism**.

Don't think of it as equivalent to `From`/`Into`.

---

# 3. `From` / `Into` — semantic type conversion

Your example:

```rust
let n: i32 = 10;
let x: i64 = n.into();
```

Conceptually:

```rust
trait Into<T> {
    fn into(self) -> T;
}
```

But you generally implement **`From`**, not `Into`:

```rust
impl From<MyType> for OtherType {
    fn from(value: MyType) -> Self {
        ...
    }
}
```

and Rust gives you the corresponding `Into` implementation.

This gives you the nice relationship:

```text
From<A> for B
       │
       └──────► A: Into<B>
```

A useful idiom is:

```rust
let x: B = a.into();
```

when the destination type is obvious from context.

Or:

```rust
let x = B::from(a);
```

when you want to make the destination explicit.

---

# 4. `TryFrom` / `TryInto` — fallible conversion

This is **very important**.

`From` means the conversion should be infallible.

But sometimes:

```text
A ────────?───────> B
```

can fail.

That's where:

```rust
TryFrom
TryInto
```

come in.

For example:

```rust
let n: i64 = 1000;

let x: u8 = n.try_into()?;
```

The conceptual trait is:

```rust
trait TryFrom<T>: Sized {
    type Error;

    fn try_from(value: T) -> Result<Self, Self::Error>;
}
```

So:

```text
From
A ─────────────> B

TryFrom
A ─────────────> Result<B, Error>
```

This distinction is fundamental:

```rust
From<T>
```

means:

> This conversion cannot fail.

whereas:

```rust
TryFrom<T>
```

means:

> This conversion may fail, and I want that represented in the type system.

---

# 5. `AsRef` / `AsMut` — borrowing conversions

These are another major category that I think is worth knowing early.

```rust
fn print_name<T: AsRef<str>>(name: T) {
    println!("{}", name.as_ref());
}
```

Now you can potentially pass:

```rust
print_name("hello");
print_name(String::from("hello"));
```

The important difference is that this isn't really:

```text
String → str
```

in the ownership/conversion sense.

It's more like:

```text
T
│
│ borrow as
▼
&Target
```

The trait is conceptually:

```rust
trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}
```

So:

```rust
String
   │
   │ as_ref()
   ▼
 &str
```

This is extremely common in generic Rust APIs.

---

# 6. `Deref` coercion — the weird one you absolutely should understand

This is another foundational Rust mechanism because it happens **implicitly**.

For example:

```rust
let s = String::from("hello");

let x: &str = &s;
```

You wrote:

```rust
&s
```

which is technically:

```rust
&String
```

but Rust allows it to become:

```rust
&str
```

through deref coercion.

Conceptually:

```text
&String
   │
   │ Deref
   ▼
&str
```

because `String` implements:

```rust
Deref<Target = str>
```

This is one of the reasons Rust code can look like it is doing mysterious conversions when it actually isn't calling `From`/`Into`.

---

# 7. Unsizing coercion

This connects directly to your previous `?Sized` question.

For example:

```rust
let x: &str = &String::from("hello");
```

and:

```rust
let x: &[i32] = &[1, 2, 3];
```

There are special **unsizing coercions**.

You can go:

```text
&T
 │
 ▼
&dyn Trait
```

when `T: Trait`, and:

```text
&[T; N]
 │
 ▼
&[T]
```

as well.

This is not `From`.

It's a language-level coercion.

---

# 8. `IntoIterator` — conversion into iteration

This isn't usually classified as a "type conversion" in the same way, but conceptually it's worth learning because Rust uses this pattern everywhere:

```rust
let xs = vec![1, 2, 3];

for x in xs {
    ...
}
```

`Vec<T>` can be converted into an iterator:

```text
Vec<T>
  │
  │ IntoIterator
  ▼
IntoIter<T>
```

And there are actually different implementations for:

```rust
Vec<T>
&Vec<T>
&mut Vec<T>
```

This becomes a major part of understanding ownership.

---

# The bigger picture I'd memorize

I'd make this little taxonomy:

```text
                    Rust "conversion"
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
    Inference          Coercion          Explicit
        │                  │                  │
     literals        deref coercion       as
     generics        unsizing
                          │
                    ┌─────┴─────┐
                    │           │
                 &String       &[T; N]
                    ↓             ↓
                   &str          &[T]

                           Explicit traits
                                │
                 ┌──────────────┼──────────────┐
                 │              │              │
               From          TryFrom        AsRef
                 │              │              │
              Into          TryInto        AsMut
```

And then there's a very useful rule of thumb:

### `From`

```text
A → B
```

**infallible ownership conversion**

### `TryFrom`

```text
A → Result<B, E>
```

**fallible ownership conversion**

### `AsRef`

```text
A → &B
```

**borrow/view conversion**

### `as`

```text
A as B
```

**explicit primitive/reference/pointer cast where Rust permits it**

### coercion

```text
A → B
```

**compiler performs it implicitly in specific language-defined situations**

### inference

```text
A ?→ B
```

**there wasn't necessarily a conversion at all; the compiler figured out what `A` should have been.**

---

And honestly, **your original `let n = 10; let x: i64 = n;` example is a very good thing to keep in your Rust notes**. It teaches a surprisingly deep concept: **the type you see in rust-analyzer's hover isn't necessarily the type that was established at the declaration site independently of the rest of the program.** Rust's type inference is global over the relevant expression/body constraints.

If you're building out a Rust fundamentals section, I'd put **inference → coercion → `as` → `From`/`Into` → `TryFrom`/`TryInto` → `AsRef`/`AsMut` → `Deref` coercion → unsizing** in roughly that progression.****
