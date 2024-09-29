FROM rust:latest

COPY . .

# Build Rust
RUN cargo build --release

# Run the app
ENTRYPOINT ./target/release/ddf_bot