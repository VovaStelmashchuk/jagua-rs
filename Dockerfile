# Use the official Rust image as a base image
FROM rust:latest

# Create a new directory for the project and set it as the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml files first to leverage Docker cache
COPY ./jagua-rs/Cargo.toml ./jagua-rs/Cargo.toml
COPY ./lbf/Cargo.toml ./lbf/Cargo.toml

# Create an empty main file to build dependencies first
RUN mkdir -p ./jagua-rs/src ./lbf/src
RUN echo "fn main() {}" > ./jagua-rs/src/main.rs
RUN echo "fn main() {}" > ./lbf/src/main.rs

# Build the dependencies only
RUN cargo build --release

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

EXPOSE 3030

# Command to run the application
CMD ["./lbf/target/release/lbf"]