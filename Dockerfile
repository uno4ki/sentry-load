FROM rust:slim-buster as builder

COPY ./ ./

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config ca-certificates libssl-dev
    
RUN cargo build

FROM debian:stable-slim
COPY --from=builder /target/debug/sentry_load /bin/sentry_load

ENTRYPOINT [ "/bin/sentry_load" ]