# Estate — Long-Term Cross-Platform Pipeline Roadmap

## 1. Platform and Environment Pipelines

- [ ] Detect the host OS: Windows, macOS, Linux.
- [ ] Detect the execution environment: native, WSL, VM, container, or remote.
- [ ] Discover OS capabilities: accessibility, input hooks, notifications, window management, tray support.
- [ ] Discover installed applications, versions, and executable paths.
- [ ] Discover home, config, cache, data, and temporary directories.
- [ ] Resolve native and virtual filesystem paths, including WSL ↔ Windows paths.
- [ ] Discover running services, daemons, and existing Estate instances.
- [ ] Detect permissions and request access where necessary.
- [ ] Normalize platform-specific errors into common error types.
- [ ] Support platform-specific implementations behind common Rust interfaces.

---

## 2. Application Integration Pipelines

### Application Families

#### IDEs and Editors

- [ ] VS Code
- [ ] Zed
- [ ] JetBrains / RustRover
- [ ] Vim / Neovim
- [ ] Other editors

#### Browsers

- [ ] Chrome
- [ ] Firefox
- [ ] Edge
- [ ] Safari

#### File Managers

- [ ] Finder
- [ ] Windows Explorer
- [ ] Linux file managers

#### Terminals

- [ ] Windows Terminal
- [ ] iTerm2
- [ ] Ghostty
- [ ] Other terminal emulators

### Application Integration

- [ ] Identify the focused application, window, tab, and document.
- [ ] Identify application capabilities and available integration methods.
- [ ] Integrate with IDE extensions, browser extensions, native APIs, and LSP.
- [ ] Read application configuration and keybindings.
- [ ] Detect application-specific actions, commands, and menus.
- [ ] Discover open projects, workspaces, tabs, and documents.
- [ ] Translate application-specific concepts into Estate's common vocabulary.
- [ ] Monitor application launches, exits, focus changes, and updates.
- [ ] Fall back to accessibility APIs or simulated shortcuts when no direct integration exists.
- [ ] Maintain compatibility adapters for different application versions.

---

## 3. Input and Keybinding Pipelines

### Keyboard

- [ ] Capture keyboard key-down and key-up events.
- [ ] Normalize Windows Alt / macOS Option.
- [ ] Normalize Windows Ctrl / macOS Command.
- [ ] Support left/right modifier distinctions.
- [ ] Support held keys.
- [ ] Support key sequences.
- [ ] Support key chords.
- [ ] Support double-taps.
- [ ] Support configurable timeouts.
- [ ] Support non-modifier prefix chords.
- [ ] Detect and recover from lost key-up events.
- [ ] Detect and recover from stuck modifiers.
- [ ] Prevent synthetic input from recursively triggering Estate.

### Mouse

- [ ] Capture mouse buttons.
- [ ] Capture vertical scrolling.
- [ ] Capture horizontal scrolling.
- [ ] Support modifier + scroll combinations.
- [ ] Support modifier + click combinations.
- [ ] Support configurable hold behavior.

### Trackpad / Gestures

- [ ] Capture supported trackpad gestures.
- [ ] Support one-finger tap / hold.
- [ ] Support two-finger scrolling.
- [ ] Support tap + scroll combinations.
- [ ] Support hold + scroll combinations.
- [ ] Support gesture sequences.
- [ ] Support configurable gesture thresholds.

### Keybinding Resolution

- [ ] Resolve global keybindings.
- [ ] Resolve application-specific keybindings.
- [ ] Resolve project-specific keybindings.
- [ ] Resolve workspace-specific keybindings.
- [ ] Detect conflicts with OS-reserved shortcuts.
- [ ] Detect conflicts with application shortcuts.
- [ ] Allow keybinding overrides.
- [ ] Support temporarily disabling or suspending global input hooks.

---

## 4. Configuration and Filesystem Pipelines

This is where the generic `FsWalker` and tiered settings resolver fit.

### Configuration Discovery

- [ ] Discover built-in defaults.
- [ ] Discover user/global configuration.
- [ ] Discover project configuration.
- [ ] Discover workspace configuration.
- [ ] Discover application-specific configuration.
- [ ] Discover configuration schemas.
- [ ] Associate configuration files with their logical layer.
- [ ] Support arbitrary configuration base names.

### Configuration Resolution

- [ ] Validate configuration against its schema.
- [ ] Merge configuration layers according to explicit precedence.
- [ ] Support partial overrides.
- [ ] Support configurable merge strategies.
- [ ] Track the source of every resolved setting.
- [ ] Support environment-variable overrides.
- [ ] Support command-line overrides.
- [ ] Watch configuration files for changes.
- [ ] Reload configuration when files change.
- [ ] Handle malformed configuration.
- [ ] Handle missing configuration.
- [ ] Handle filesystem permission errors.

### Filesystem

- [ ] Resolve Windows paths.
- [ ] Resolve macOS paths.
- [ ] Resolve Linux paths.
- [ ] Resolve WSL paths.
- [ ] Resolve Windows ↔ WSL paths.
- [ ] Resolve symlinks where appropriate.
- [ ] Resolve relative paths.
- [ ] Normalize paths into Estate's internal representation.
- [ ] Track which environment owns a path.
- [ ] Support local and remote filesystem providers.

---

## 5. Context and Semantic Discovery Pipelines

- [ ] Track the currently focused application.
- [ ] Track the focused window.
- [ ] Track the active document.
- [ ] Track the active project.
- [ ] Track the active workspace.
- [ ] Track the active repository.
- [ ] Discover the current selection.
- [ ] Discover cursor position.
- [ ] Discover surrounding document content.
- [ ] Build document outlines.
- [ ] Build semantic navigation targets.
- [ ] Discover IDE symbols through LSP.
- [ ] Discover browser headings and page structure.
- [ ] Discover filesystem hierarchy.
- [ ] Detect active terminal sessions.
- [ ] Detect terminal working directories.
- [ ] Maintain an application-independent representation of navigation targets.
- [ ] Distinguish stale context from current context.
- [ ] Expose context through the Estate daemon.

---

# 6. Intent and Action Pipelines

This is the center of Estate.

**Inputs describe how an action was requested; intents describe what should happen.**

| Intent               | Possible implementations                                    |
| -------------------- | ----------------------------------------------------------- |
| `OpenOutline`        | IDE symbol outline, browser headings, file tree             |
| `OpenCommandPalette` | Native command palette, Estate overlay                      |
| `NavigateBack`       | Editor location history, browser history, folder history    |
| `NavigateForward`    | Editor location history, browser history, folder history    |
| `FindSymbol`         | LSP, IDE API, document parser                               |
| `OpenFile`           | IDE, Finder, Explorer, file manager                         |
| `RevealCurrentFile`  | Finder, Explorer, Linux file manager                        |
| `CopyLastOutput`     | Terminal integration, captured command output               |
| `SearchEverywhere`   | IDE search, browser search, filesystem search, Estate index |
| `OpenProject`        | IDE, file manager, terminal                                 |
| `OpenWorkspace`      | IDE workspace, Estate workspace                             |
| `SwitchWorkspace`    | IDE workspace, browser profile, Estate workspace            |
| `OpenSettings`       | Application settings, Estate settings                       |
| `OpenKeybindings`    | Application keybindings, Estate keybindings                 |
| `GoToDefinition`     | LSP, IDE API                                                |
| `GoToReference`      | LSP, IDE API                                                |
| `GoToImplementation` | LSP, IDE API                                                |
| `RenameSymbol`       | LSP, IDE API                                                |
| `FormatDocument`     | LSP, formatter, IDE API                                     |
| `RunCommand`         | IDE command, terminal command, Estate command               |
| `OpenTerminal`       | IDE terminal, native terminal                               |
| `FocusApplication`   | OS window API                                               |
| `FocusWindow`        | OS window API                                               |
| `MinimizeWindow`     | OS window API                                               |
| `MaximizeWindow`     | OS window API                                               |
| `CloseWindow`        | OS window API                                               |
| `MoveWindow`         | OS window API                                               |
| `Copy`               | Application API, synthetic keyboard input                   |
| `Paste`              | Application API, synthetic keyboard input                   |
| `Undo`               | Application API, synthetic keyboard input                   |
| `Redo`               | Application API, synthetic keyboard input                   |

### Action Architecture

- [ ] Define a platform-independent action registry.
- [ ] Define typed inputs for every action.
- [ ] Define typed outputs for every action.
- [ ] Define structured errors for every action.
- [ ] Map keyboard shortcuts to actions.
- [ ] Map gestures to actions.
- [ ] Map CLI commands to actions.
- [ ] Map UI buttons to actions.
- [ ] Map application events to actions.
- [ ] Resolve actions against available capabilities.
- [ ] Select an execution adapter.
- [ ] Support action composition.
- [ ] Support action sequences.
- [ ] Support cancellation.
- [ ] Support action timeouts.
- [ ] Distinguish read-only actions from mutating actions.
- [ ] Support undo/rollback where possible.

---

# 7. Capability and Execution Pipelines

An intent should not know _how_ it is implemented.

For example:

```text
OpenOutline
    ↓
Capability Resolver
    ↓
┌──────────────────────────────┐
│ IDE API                      │
│ LSP                          │
│ Browser Extension            │
│ Accessibility API            │
│ Synthetic Input              │
│ Estate Overlay               │
└──────────────────────────────┘
    ↓
Execution
```
