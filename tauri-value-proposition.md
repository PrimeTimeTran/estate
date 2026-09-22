Yeah — I think the landing page should **not** sell “Rust is faster.” That's true in the right workload, but it's too generic and actually undersells what you're building.

Your sharper proposition is:

> **Write desktop-native tools with the technologies you already know — HTML, CSS, and TypeScript — while moving the expensive/native capabilities underneath into Rust.**

And yes, your Electron intuition is directionally right, but there's an important distinction: **Electron's weight is not simply “JavaScript is slow.”** Electron bundles Chromium + Node.js, whereas Tauri uses the OS WebView. Electron's own docs emphasize that performance depends heavily on what the app does and how it is profiled. ([Electron][1])

Also, the VS Code memory story you remember is probably conflating a few things. VS Code itself is heavily optimized and deliberately moves work into separate processes; Microsoft explicitly discusses moving expensive work out of the Electron main process because blocking it hurts UI responsiveness. ([Visual Studio Code][2]) There are also current projects demonstrating the architecture you're imagining: SideX is rebuilding the VS Code workbench with Tauri/Rust while retaining the TypeScript UI, and DSCode is doing a Rust-core + Node extension-host architecture. ([GitHub][3])

So if I were writing the **six big tabs/cards** at the top of your landing page, I'd make them about these:

---

### 01 — **Web UI**

**Build native apps with HTML, CSS & TypeScript.**

```text
HTML
CSS
TypeScript
      ↓
Estate
      ↓
Native macOS / Windows / Linux
```

No custom UI framework to learn. Your existing web skills become desktop UI.

---

### 02 — **Native Core**

**Put the heavy work where it belongs.**

Filesystem. Processes. Search. Parsing. Indexing. Git. Networking. AI agents. Large datasets.

```text
Web UI
   ↕
typed bridge
   ↕
Rust
```

Your UI stays expressive and familiar while computation and system integration can run in native code.

And I would phrase this as **“native performance”**, not “Rust is always faster.” That's much more defensible.

---

### 03 — **OS Access**

**Give web developers capabilities the browser can't provide.**

This is potentially your killer differentiator.

```ts
await estate.context.focusedApp()
await estate.project.current()
await estate.fs.read(...)
await estate.process.spawn(...)
await estate.clipboard.read()
```

A normal website cannot simply do those things.

A Node application can do _some_ of them, of course — but now you're providing a **controlled native capability layer** rather than asking every developer to build their own Node/native integration.

And eventually:

```ts
await estate.context.focusedElement()
```

could resolve through your macOS Accessibility integration.

That's genuinely interesting.

---

### 04 — **Native Everywhere**

**One UI. Native runtime. Every desktop.**

Don't sell:

> “Rust replaces JavaScript.”

Sell:

> **“Your web skills, without giving up the operating system.”**

```text
             Your App
                │
       ┌────────┴────────┐
       │                 │
    Web UI          Native APIs
       │                 │
 HTML/CSS/TS           Rust
       │                 │
       └────────┬────────┘
                │
        macOS / Windows / Linux
```

This also distinguishes you from something like Electron: Electron provides a consistent Chromium runtime; Tauri uses the OS webview, which can substantially change the resource/runtime tradeoff. SideX is a contemporary example of this exact Electron → Tauri architectural trade. ([GitHub][3])

---

### 05 — **Context**

**Your app knows what's happening around it.**

This is where Estate becomes much more than another Tauri wrapper.

```ts
const context = await estate.context.current()
```

Could eventually tell you:

```text
Focused app
Focused window
Focused element
Current URL
Current document
Selection
Project
Workspace
Terminal
Process
Agent
```

That's a very different primitive from:

```ts
fs.readFile(...)
```

You're giving a web developer access to **the user's computing context**.

And _this_ is where your AX work starts becoming product-defining rather than just an implementation detail.

---

### 06 — **Compose**

**Build tools that plug into the user's environment.**

This is the one I'd make the most ambitious.

```text
HTML/CSS/TS component
          ↓
      Estate SDK
          ↓
 ┌────────┼─────────┐
 │        │         │
Project Context   Agents
 │        │         │
Files    Apps     Processes
```

A developer can make:

- a global sidebar
- a project dashboard
- a dependency inspector
- an agent console
- a browser companion
- a terminal UI
- a command palette
- a contextual popup
- a LeetCode helper
- whatever weird little tool they want

without becoming a Rust developer.

---

## The six words might literally be

```text
WEB UI       NATIVE CORE       OS ACCESS
CONTEXT      NATIVE RUNTIME    COMPOSE
```

Or, if you want it more marketing-y:

```text
BUILD        ACCELERATE        CONNECT
CONTROL      UNDERSTAND        EXTEND
```

But I actually prefer the first version because **it tells the technical story**.

And there's one thing I'd put prominently above all six:

> ### **The native runtime for web developers.**
>
> Build with HTML, CSS, and TypeScript. Access the filesystem, processes, projects, applications, agents, and operating system through a native Rust core.

That gets much closer to what I think you're actually discovering.

Because **“Rust is faster” isn't the product.**

The product is:

> **“I already know how to build web applications. Now my application can participate in the desktop.”**

And _then_ Rust is the mechanism that makes the native half powerful.

[1]: https://www.electronjs.org/docs/latest/tutorial/performance?utm_source=chatgpt.com 'Performance | Electron'
[2]: https://code.visualstudio.com/blogs/2022/11/28/vscode-sandbox?utm_source=chatgpt.com 'Migrating VS Code to Process Sandboxing'
[3]: https://github.com/Sidenai/sidex?utm_source=chatgpt.com 'GitHub - Sidenai/sidex: VS Code rebuilt on Tauri. Same architecture, 96% smaller. Early release. · GitHub'
