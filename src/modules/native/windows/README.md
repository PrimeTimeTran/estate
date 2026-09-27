| Layer                         | Use                      | Role                                                                            |
| ----------------------------- | ------------------------ | ------------------------------------------------------------------------------- |
| **Windows**                   | Native OS                | The actual product/runtime environment                                          |
| **PowerShell**                | Primary Windows shell    | Windows-native builds, installers, services, registry, signing, packaging       |
| **WSL**                       | Linux dev environment    | Linux builds, Unix tooling, CI-like checks, scripts, cross-platform development |
| **Git Bash**                  | Compatibility shell      | Occasional Unix commands / existing shell scripts                               |
| **Git**                       | Shared source control    | Same repo, same history                                                         |
| **Zed / VS Code / RustRover** | IDE                      | Can work against either environment                                             |
| **Cargo**                     | Build system             | Platform-specific target/toolchain                                              |
| **Windows SDK/MSVC**          | Native Windows toolchain | Actual Windows binaries                                                         |
| **WSL Ubuntu**                | Linux toolchain          | Actual Linux binaries                                                           |

The important distinction is:

> **WSL is a development environment. Windows is a target platform.**

Don't make WSL pretend to be Windows.

---

# What I would build for Estate

I'd structure your development matrix like this:

```text
                         ESTATE
                           │
              ┌────────────┴────────────┐
              │                         │
          PLATFORM                     CORE
              │                         │
      ┌───────┼────────┐                │
      │       │        │                │
    Windows  macOS    Linux             │
      │       │        │                │
    MSVC    Apple    WSL/Linux          │
      │       │        │                │
      └───────┼────────┘                │
              │                         │
              └────────── Cargo ────────┘
```

Then your repo has explicit commands:

```text
cargo check-linux
cargo check-windows
cargo check-macos

cargo test-linux
cargo test-windows
cargo test-macos

cargo build-linux
cargo build-windows
cargo build-macos
```

No implicit assumption that the machine you're currently sitting in is the target.

---

# PowerShell should be your Windows "truth"

For native Windows development, I'd make **PowerShell the canonical Windows shell**.

For example:

```powershell
cargo check
cargo test
cargo build
cargo run --bin platform
```

This is where you exercise:

- Windows filesystem semantics
- Windows environment variables
- Windows process behavior
- Windows services
- Windows registry
- Windows named pipes
- Windows native GUI
- Windows notifications
- Windows input hooks
- Windows installers
- Windows code signing
- Windows Defender behavior
- Windows permissions
- Windows startup
- Windows Task Scheduler
- Windows paths
- Windows shell integration

That's the environment that matters for a Windows application.

---

# WSL should be your Linux laboratory

Your current:

```text
/home/prime/kb/project
```

is perfect for Linux-side development.

There you should be able to do:

```bash
cargo check
cargo test
cargo fmt
cargo clippy
cargo tree
git
rg
fd
awk
sed
jq
bash
python
```

and build:

```text
x86_64-unknown-linux-gnu
```

That gives you a genuine Linux environment rather than a simulated Windows one.

---

# Git Bash is optional

I wouldn't make Git Bash part of the architecture.

Think of it as:

```text
Git Bash
   ↓
compatibility/convenience shell
```

rather than:

```text
Git Bash
   ↓
third development environment
```

If a script works in Git Bash but breaks in PowerShell, that's not particularly useful for a Windows-native product.

For Estate, I'd prefer:

```text
scripts/
    build.ps1
    build.sh
```

with the platform-specific behavior explicit.

Or better, eventually:

```text
cargo xtask ...
```

so your actual developer workflow becomes platform-neutral:

```text
cargo xtask check
cargo xtask build
cargo xtask package
cargo xtask install
```

and `xtask` decides what needs to happen on each OS.

---

# The filesystem is where this gets interesting

I would **not** try to make Windows and WSL share the exact same working directory.

Instead:

```text
Windows:
C:\Users\<you>\src\estate

WSL:
\\wsl$\Ubuntu\home\prime\src\estate
```

Each filesystem should belong to its OS.

For heavy Rust development, keep the Linux checkout **inside the WSL filesystem**, not under:

```text
/mnt/c/...
```

Likewise, keep the native Windows checkout on NTFS.

That avoids a huge class of filesystem/performance/permission weirdness.

---

# But you don't necessarily need two permanent checkouts

You have another option that I think fits your Estate work particularly well:

```text
                    Git remote
                       │
              ┌────────┴────────┐
              │                 │
             WSL              Windows
              │                 │
         Linux checkout    Windows checkout
              │                 │
        Linux development   Windows validation
```

Both are disposable working trees of the same repository.

You can use:

```text
WSL
  ↓
primary development
  ↓
git commit
  ↓
Windows
  ↓
native Windows validation
```

This is actually **much cleaner** than trying to make WSL's filesystem magically act like Windows.

---

# I'd go one step further for Estate

Make the project understand its platform explicitly.

Something like:

```text
cargo xtask doctor
```

would report:

```text
Estate development environment
────────────────────────────────

Host OS:          Windows
Architecture:     x86_64
Rust:             1.98.1
Toolchain:        stable-x86_64-pc-windows-msvc
Windows SDK:      OK
MSVC:             OK
Git:              OK
Cargo:            OK

Targets
────────────────────────────────
Windows           ✓
Linux             ✓ / WSL
macOS             —
WASM              ✓

Native dependencies
────────────────────────────────
protoc            ✓
LLVM              ✓
Node              ✓
...
```

On WSL:

```text
Estate development environment
────────────────────────────────

Host OS:          Linux
Environment:      WSL2
Architecture:     x86_64
Rust:             1.98.1
Toolchain:        stable-x86_64-unknown-linux-gnu

Targets
────────────────────────────────
Linux             ✓
Windows           ?
macOS             —
WASM              ✓
```

Now you can immediately see **what environment you're actually testing**.

---

# And then make `platform` a real matrix

Your current smoke test is actually the beginning of something useful.

I'd eventually have:

```text
cargo xtask smoke input
```

produce:

### Windows

```text
Input backend: Windows
──────────────────────────────
Keyboard hook       ✓
Mouse hook          ✓
Mouse X1            ✓
Mouse X2            ✓
Mouse wheel         ✓
Global input        ✓
```

### macOS

```text
Input backend: macOS
──────────────────────────────
CGEventTap          ✓
Keyboard            ✓
Mouse               ✓
Accessibility       ✓
```

### Linux

```text
Input backend: Linux
──────────────────────────────
evdev               ✓
libinput            ✓
Keyboard            ✓
Mouse               ✓
```

That is much more valuable than merely asking whether `cargo build` succeeded.

---

# For your specific Windows goal

I'd organize the entire development stack as:

```text
                 ┌─────────────────────────┐
                 │        ESTATE            │
                 │     cross-platform       │
                 └────────────┬────────────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
                Shared Core         Platform
                    │                   │
                    │       ┌───────────┼───────────┐
                    │       │           │           │
                    │     Windows      macOS       Linux
                    │       │           │           │
                    │     MSVC        Apple       WSL
                    │       │           │           │
                    └───────┴───────────┴───────────┘
```

And around that:

```text
                 Developer Experience
                         │
          ┌──────────────┼──────────────┐
          │              │              │
        Zed           PowerShell       WSL
          │              │              │
          └──────────────┼──────────────┘
                         │
                       Cargo
                         │
                    cargo xtask
                         │
        ┌────────────────┼────────────────┐
        │                │                │
     Check             Test             Package
        │                │                │
        └────────────────┼────────────────┘
                         │
                    Git / CI
```

That gives you a much more robust philosophy:

> **Don't make one environment capable of pretending to be every OS. Make every OS a first-class environment, and make the project tooling unify them.**

For what you're trying to do with Estate, I'd make **PowerShell + native MSVC Windows the authoritative Windows development environment**, **WSL the authoritative Linux environment**, and use **Cargo/xtask as the abstraction layer above both**. Git Bash can remain a convenience tool rather than something your architecture depends on.
