# syntax=docker/dockerfile:1
FROM rust:1.88-slim-bookworm AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

FROM base AS manifest

WORKDIR /app

# Copy workspace files and source
COPY services ./services

# Generate services manifest
RUN ls -1 services > /app/services-manifest.txt
RUN cat /app/services-manifest.txt

FROM base AS dev

# Copy services manifest
COPY --from=manifest /app/services-manifest.txt ./app/services-manifest.txt

# Install cargo-watch for development
RUN cargo install cargo-watch

WORKDIR /workspace

CMD ["cargo", "watch", "-x", "run"]

FROM base AS builder
ARG SERVICE
ARG FEATURES=""

WORKDIR /app

# Copy workspace files and source
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY crates ./crates
COPY services ./services

# Build target service with optional features
RUN if [ -z "$FEATURES" ]; then \
  cargo build --release --package ${SERVICE}; \
else \
  cargo build --release --package ${SERVICE} --features ${FEATURES}; \
fi && \
cp target/release/${SERVICE} /app/binary

FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/binary /usr/local/bin/app

# Copy services manifest
COPY --from=manifest /app/services-manifest.txt ./app/services-manifest.txt

# Create and use non-root user
RUN useradd -r -s /bin/false appuser
USER appuser

ENTRYPOINT ["/usr/local/bin/app"]
