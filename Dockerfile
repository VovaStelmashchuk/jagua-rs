FROM rust:1.54 as builder

COPY . /app

WORKDIR /app/lbf

RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /app/lbf/target/release/lbf /usr/local/bin

CMD ["lbf"]