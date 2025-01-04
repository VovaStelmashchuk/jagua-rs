FROM rust:1.83-slim-bullseye AS builder

RUN apt-get update && apt-get install -y build-essential && rm -rf /var/lib/apt/lists/*

ENV USER=root

WORKDIR /usr/src/jagua-rs

COPY Cargo.toml ./
COPY jagua-rs/Cargo.toml jagua-rs/
COPY lbf/Cargo.toml lbf/
COPY server/Cargo.toml server/
COPY arr_macro_impl/Cargo.toml arr_macro_impl/

# Create dummy source files to build dependencies
RUN mkdir -p jagua-rs/src lbf/src server/src arr_macro_impl/src && \
    echo "pub fn jagua_dummy() {}" > jagua-rs/src/lib.rs && \
    echo "pub fn lbf_dummy() {}" > lbf/src/lib.rs && \
    echo "fn main() { println!(\"Dummy server running...\"); }" > server/src/main.rs && \
    echo "// Dummy procedural macro" > arr_macro_impl/src/lib.rs

RUN cargo build --release

# Remove dummy source files
RUN rm jagua-rs/src/lib.rs lbf/src/lib.rs server/src/main.rs arr_macro_impl/src/lib.rs

# Copy actual source code
COPY jagua-rs/src jagua-rs/src
COPY lbf/src lbf/src
COPY server/src server/src
COPY arr_macro_impl/src arr_macro_impl/src

RUN cargo build --release -p server

# Stage 2: Create the final minimal image
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/jagua-rs/target/release/server /usr/local/bin/server

RUN chmod +x /usr/local/bin/server

EXPOSE 3030

CMD ["/usr/local/bin/server"]
