$ cargo build -p estate

lsof -nP -iTCP:50051 -sTCP:LISTEN

cargo run --bin server
