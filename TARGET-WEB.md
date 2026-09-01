# Trouble shooting WASM

The LSP, IDE, Cargo.toml and CLI args need to match up for builds to work right.

Not to mention feature flags

```sh
# You have successful native build
cargo run --bin native --features=native
# You want successful web build
cargo run --bin web --features=web
# You must have this in Cargo.toml
default = ["native"]
```

```sh
# Cargo.toml
default = ["web"]
# For this command to succeed
cargo check \
  --no-default-features \
  --target wasm32-unknown-unknown \
  --features web
```

```sh
default = ["native"]
```

## Command

```sh
$ cargo check \
    --target wasm32-unknown-unknown \
    --features web
```

## Debug WASM Target for IDE

Set Rust Analayzer in VSCode settings

```json
{
	"rust-analyzer.cargo.target": "wasm32-unknown-unknown",
	"rust-analyzer.cargo.features": ["web"]
}
```

## Cargo

Must have default features set as "web" to get correct feedback from VSCode
Also define dependencies for it.

```toml
[features]
default = ["web"]
web = [
  "dep:wasm-bindgen",
  "dep:web-sys",
  "dep:eframe",
  "dep:egui_commonmark",
  "dep:egui_extras",
  "dep:jsonc-parser",
  "dep:tonic",
]
```

## Dependencis

```sh
# What pulls mio into my native/default build?
cargo tree -i mio
# What pulls mio into the WASM dependency graph?
cargo tree -i mio --target wasm32-unknown-unknown
```

```sh
cargo tree \
  --target wasm32-unknown-unknown \
  --features web \
  -i mio

cargo tree \
  --target wasm32-unknown-unknown \
  --features web \
  -i tokio
```

```sh
default = ["native"]
```

## Review

## Zed Web build

```toml
default = ["web"]
```

```json
"lsp": {
		"rust-analyzer": {
			"initialization_options": {
				"cargo": {
					"target": "wasm32-unknown-unknown",
					"features": ["web"],
					"targetDir": true
				}
			}
		},
}
```

## Native

```sh
cargo check \
  --no-default-features \
  --features native
cargo build --bin native --features native
```

## Web

```sh
cargo check \
  --no-default-features \
  --target wasm32-unknown-unknown \
  --features web
cargo build --bin web --no-default-features --features="web" --target wasm32-unknown-unknown
trunk build --release
trunk serve src/web/public/index.html --features web
```

```
protoc \
  -I proto \
  --descriptor_set_out=/tmp/test.pb \
  proto/main.proto
```
