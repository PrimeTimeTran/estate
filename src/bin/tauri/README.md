# Tauri + Vanilla TS

This template should help get you started developing with Tauri in vanilla HTML, CSS and Typescript.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Install

- cargo install tauri-cli

## Running

- App but be it's own workspace

```sh
# Cargo.toml
[workspace]

[package]
name = "servo-tauri-app"
version = "0.1.0"
```

- `cargo tauri dev`
  Run app successfully if the app is it's own workspace
