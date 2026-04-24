FROM rust:1.80-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    cargo build --release && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/bendy-web-sential /app/bendy-web-sential
COPY --from=builder /app/.env.example /app/.env 2>/dev/null || true

ENV RUST_LOG=info
ENV BWS_DATABASE_URL=sqlite:///data/bws.db
ENV BWS_GATEWAY_PORT=8080
ENV BWS_ADMIN_PORT=3000

EXPOSE 8080 3000

VOLUME ["/data"]

ENTRYPOINT ["/app/bendy-web-sential"]
