cargo build --bin bootstrap
cargo install --path . --bin bootstrap
bootstrap --manifest ../estate/1-estate-workspace-with-persona.md -f 1
export PATH="$PWD/target/debug:$PATH"

cargo build --bin bootstrap && cargo install --bin bootstrap --path .

1. Static
- Exactly the same for every project.
2. Rendered
- Contain project-specific values.
3. Generated files
-
