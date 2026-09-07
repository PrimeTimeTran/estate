// $ cargo build --bin web --no-default-features --features="web" --target wasm32-unknown-unknown
// $ trunk serve src/web/public/index.html --features web
//
// cargo build \
//   --target wasm32-unknown-unknown \
//   --bin web \
//   --no-default-features \
//   --features web
//
//   cargo build --bin web --no-default-features --features="web" --target wasm32-unknown-unknown && trunk serve src/web/public/index.html --features web
//
// Debug Build
// cargo tree \
//   --bin web \
//   --target wasm32-unknown-unknown \
//   -e features
//
// "rust-analyzer.cargo.target": "wasm32-unknown-unknown",

#[cfg(all(feature = "web", target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
	// Build and run
	// cargo build --bin web --target wasm32-unknown-unknown --features web && trunk serve src/web/public/index.html
	Ok(())
}

fn _troubleshoot_wasm() {
	// wasm-objdump -x ../../target/wasm32-unknown-unknown/release/web.wasm | grep -i clone_ref
	// cargo tree --target wasm32-unknown-unknown -e features -i wasm-bindgen
	// cl
	// wasm-objdump -x ../../target/wasm32-unknown-unknown/release/web.wasm | head -150
	// wasm-objdump -x ../../target/wasm32-unknown-unknown/release/web.wasm | grep -E 'producers|target_features|custom'
	// cl
	// wasm-objdump -x ../../target/wasm32-unknown-unknown/release/web.wasm | \\n  grep -A20 -B2 'producers'
	// wasm-objdump -x ../../target/wasm32-unknown-unknown/release/web.wasm | \\n  grep -A20 -B2 'target_features'
	// wasm-objdump -x ../../target/wasm32-unknown-unknown/release/web.wasm | \\n  grep -A50 -B2 'name: "producers"'
	// wasm-bindgen \\n  --target web \\n  --out-dir ../../target/wasm-bindgen-debug-test \\n  --out-name web \\n  ../../target/wasm32-unknown-unknown/debug/web.wasm \\n  --no-typescript
	// wasm-bindgen \\n  --target web \\n  --out-dir ../../target/wasm-bindgen-test \\n  --out-name web \\n  ../../target/wasm32-unknown-unknown/debug/web.wasm \\n  --no-typescript \\n  --keep-debug
	// cargo tree --target wasm32-unknown-unknown -p wasm-bindgen
	// cargo tree --target wasm32-unknown-unknown -p wasm-bindgen-macro
	// cargo tree --target wasm32-unknown-unknown -e features -i wasm-bindgen
	// wasm-tools objdump ../../target/wasm32-unknown-unknown/debug/web.wasm
	// wasm-objdump -x ../../target/wasm32-unknown-unknown/debug/web.wasm | \\n  grep -A10 '^Export'
	// grep -R -n '#\[wasm_bindgen(start)\]' crates/estate crates 2>/dev/null
}
