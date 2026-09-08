## Cheatsheet

```sh
cargo +nightly check
cargo +nightly build
cargo +nightly run
cargo +nightly test

cargo +nightly check && cargo +nightly build && cargo +nightly run && cargo +nightly test

# Set this this crate
rustup override set nightly

# Now these use nightly
# rustc --version
# cargo check
# cargo build
# cargo test

rustup show
# To confirm
# active toolchain
# ----------------
# name: nightly-aarch64-apple-darwin
# active because: directory override for '/.../estate'

# Unset
rustup override unset
```

```sh
rustup toolchain install nightly --profile minimal
```

Then verify:

```bash
rustc +nightly --version
cargo +nightly --version
```

You should get something around:

```text
rustc 1.100.0-nightly (...)
```

Your stable toolchain remains exactly as it was.

---

## 2. Check what targets you currently have installed

```bash
rustup target list --installed
```

You'll probably see something like:

```text
aarch64-apple-darwin
wasm32-unknown-unknown
wasm32-wasi
```

The important thing is that **targets are installed per toolchain**.

So if you need WASM for nightly, install the target explicitly:

```bash
rustup +nightly target add wasm32-unknown-unknown
```

If what you actually need is WASI, the target naming has changed over Rust's history. For modern Rust, check:

```bash
rustc +nightly --print target-list | grep wasm
```

You'll likely see targets such as:

```text
wasm32-unknown-unknown
wasm32-wasip1
wasm32-wasip2
```

So for WASI you may want:

```bash
rustup +nightly target add wasm32-wasip1
```

rather than the old:

```text
wasm32-wasi
```

---

# The workflow I'd recommend

You **don't actually need to switch your entire machine back and forth**.

Use explicit toolchains:

### Stable / released Rust

```bash
cargo build
cargo test
cargo check
cargo run
```

### Nightly

```bash
cargo +nightly build
cargo +nightly test
cargo +nightly check
cargo +nightly run
```

This is probably the cleanest setup for you.

You can literally have two terminals:

```bash
# normal Estate development
cargo check
```

and:

```bash
# experimenting with nightly
cargo +nightly check
```

No global switching required.

---

# If you DO want to switch the default

See what you currently have:

```bash
rustup toolchain list
```

You'll get something like:

```text
stable-aarch64-apple-darwin
nightly-aarch64-apple-darwin
```

Set stable globally:

```bash
rustup default stable
```

Set nightly globally:

```bash
rustup default nightly
```

Check:

```bash
rustc --version
cargo --version
```

And switch back:

```bash
rustup default stable
```

---

# Even better: make Estate nightly locally

Since you're specifically experimenting in `estate`, I'd use a **directory override**.

From your Estate repo:

```bash
cd estate
rustup override set nightly
```

Now:

```bash
cargo check
```

automatically uses nightly **inside that repository**.

Outside Estate:

```bash
cd ..
rustc --version
```

you'll still be on stable.

Check the active override:

```bash
rustup show
```

You'll see something like:

```text
active toolchain
----------------
nightly-aarch64-apple-darwin (directory override for '/path/to/estate')
```

### Remove the override

```bash
rustup override unset
```

Then Estate goes back to whatever your global default is.

---

# My preferred setup for your project

I'd actually do this:

```bash
# 1. Install nightly
rustup toolchain install nightly --profile minimal

# 2. Make Estate use nightly
cd estate
rustup override set nightly

# 3. Verify
rustc --version
cargo --version

# 4. Work normally
cargo check
cargo test
cargo build
```

Then when you want to go back to released Rust:

```bash
rustup override unset
```

And you're back on stable.

Or, if you want to keep the project permanently pinned to nightly while occasionally testing stable:

```bash
cargo +stable check
```

and:

```bash
cargo check
```

will use nightly because of the directory override.

---

## Your "basket of commands"

I'd keep this little cheat sheet around:

```bash
# ─────────────────────────────
# INSTALL
# ─────────────────────────────

rustup toolchain install nightly --profile minimal


# ─────────────────────────────
# CHECK TOOLCHAINS
# ─────────────────────────────

rustup toolchain list
rustup show


# ─────────────────────────────
# RUN SOMETHING ON NIGHTLY
# ─────────────────────────────

cargo +nightly check
cargo +nightly build
cargo +nightly test
cargo +nightly run


# ─────────────────────────────
# RUN SOMETHING ON STABLE
# ─────────────────────────────

cargo +stable check
cargo +stable build
cargo +stable test
cargo +stable run


# ─────────────────────────────
# MAKE CURRENT PROJECT NIGHTLY
# ─────────────────────────────

rustup override set nightly

# Now normal cargo commands use nightly:
cargo check
cargo test
cargo build


# ─────────────────────────────
# RETURN CURRENT PROJECT TO DEFAULT
# ─────────────────────────────

rustup override unset


# ─────────────────────────────
# CHANGE GLOBAL DEFAULT
# ─────────────────────────────

rustup default nightly
rustup default stable


# ─────────────────────────────
# TARGETS
# ─────────────────────────────

rustup target list --installed
rustc +nightly --print target-list | grep wasm

rustup +nightly target add wasm32-unknown-unknown
rustup +nightly target add wasm32-wasip1
```

### One important distinction

`rustup update nightly` is **not necessary** every time you want to use nightly.

Once installed:

```bash
cargo +nightly check
```

uses the installed nightly.

When you deliberately want to move to a newer nightly:

```bash
rustup update nightly
```

But because you're doing serious Estate development alongside experimental compiler features, I'd strongly consider **pinning a specific nightly** rather than tracking `nightly` forever:

```bash
rustup toolchain install nightly-2026-08-18 --profile minimal
```

then:

```bash
rustup override set nightly-2026-08-18
```

That makes your compiler reproducible, which is especially useful when you're experimenting with unstable/nightly features.
