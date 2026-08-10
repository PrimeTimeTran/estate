cargo build --bin bootstrap
cargo install --path . --bin bootstrap
bootstrap --manifest ../estate/1-estate-workspace-with-persona.md -f 1
export PATH="$PWD/target/debug:$PATH"





cargo build --bin bootstrap && cargo install --bin bootstrap --path .
