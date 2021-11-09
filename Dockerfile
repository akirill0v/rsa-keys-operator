FROM rust:latest as builder
WORKDIR /usr/src

COPY . .
RUN cargo build --target x86_64-unknown-linux-gnu --release

FROM debian:stable-slim
WORKDIR /
COPY --from=builder /usr/src/target/x86_64-unknown-linux-gnu/release/key_generator .
USER 1000
ENTRYPOINT ["/key_generator"]
