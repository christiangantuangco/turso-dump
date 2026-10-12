# syntax=docker/dockerfile:1

FROM rust:1.98-slim-trixie AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
    && cp target/release/turso-dump /usr/local/bin/turso-dump

FROM debian:trixie-slim
COPY --from=builder /usr/local/bin/turso-dump /usr/local/bin/turso-dump
ENV HOME=/history
WORKDIR /data
ENTRYPOINT ["turso-dump"]
CMD ["/data/node.db"]
