ARG RUST_VERSION=1.40.0
FROM ekidd/rust-musl-builder:$RUST_VERSION-openssl11 AS build
ARG rust_args="--target x86_64-unknown-linux-musl --release"
WORKDIR /home/rust/src

RUN USER=rust cargo init . \
  && touch src/lib.rs
COPY Cargo.toml Cargo.lock ./
RUN cargo build $rust_args

# Copy the source and build the application

COPY ./src ./src
RUN sudo chown -R rust:rust ./src
RUN cargo clean -p key_generator $rust_args
RUN rm target/x86_64-unknown-linux-musl/release/deps/key_generator-*
RUN cargo build $rust_args

# Copy the statically-linked binary into a scratch container.
FROM scratch
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/key_generator .

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

USER 1000
CMD ["./key_generator"]
