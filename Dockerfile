FROM rust:slim-buster as builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config ca-certificates libssl-dev

# Special magic for cargo build
RUN mkdir -p src/ && touch src/lib.rs
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build

# Now build app
COPY ./ ./
RUN cargo build

FROM debian:stable-slim

COPY --from=builder /target/debug/sentry_load /bin/sentry_load

ENTRYPOINT [ "/bin/sentry_load" ]
