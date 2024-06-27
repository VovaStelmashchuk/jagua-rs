FROM rust:1.54 as builder

WORKDIR /app

COPY . /app

RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /app/target/release/lbf /usr/local/bin

CMD ["lbf"]