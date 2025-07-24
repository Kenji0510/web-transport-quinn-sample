# Build
cargo build --release

# Build
cargo build --release --bin server
cargo build --release --bin client

# Run
./target/release/server
./target/release/client --insecure