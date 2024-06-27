# Use the official Rust image as a base image
FROM rust:latest AS builder

# Create a new directory for the project and set it as the working directory
WORKDIR /usr/src/app

# Copy the source code
COPY . .
# build cargo for lbf and jagua-rs
RUN cargo build --release --manifest-path ./jagua-rs/Cargo.toml
RUN cargo build --release --manifest-path ./lbf/Cargo.toml

# Start a new stage to create a smaller image without unnecessary build dependencies
FROM debian:bookworm-slim

# Set the working directory
WORKDIR /usr/src/app

# Copy the built binary from the previous stage
COPY --from=builder /usr/src/app/lbf/target/release/lbf .

# Command to run the application
CMD ["./lbf"]