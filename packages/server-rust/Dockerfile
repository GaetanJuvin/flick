FROM rust:1.82 AS builder

WORKDIR /app

# Copy manifests first for dependency caching
COPY packages/server-rust/Cargo.toml packages/server-rust/Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy source and migrations
COPY packages/server-rust/src ./src
COPY packages/server/src/db/migrations ./migrations

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/flick-server /usr/local/bin/flick-server
COPY --from=builder /app/migrations /app/migrations

WORKDIR /app
EXPOSE 3001

CMD ["flick-server"]
