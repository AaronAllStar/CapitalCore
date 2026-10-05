# syntax=docker/dockerfile:1.4
# Multi-stage production build for EdgeArena Rust Core Decision Platform

# --- Stage 1: Build stage ---
FROM rust:1-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifest and source trees
COPY Cargo.toml Cargo.lock rustfmt.toml clippy.toml deny.toml ./
COPY crates ./crates
COPY ml ./ml
COPY golden ./golden

# Compile release binary for edge-api
RUN cargo build --release --bin edge-api

# Strip symbols to minimize binary footprint
RUN strip /app/target/release/edge-api

# --- Stage 2: Minimal Distroless Runtime ---
FROM gcr.io/distroless/cc-debian11:nonroot AS runtime

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/edge-api /usr/local/bin/edge-api

# Expose HTTP API port
EXPOSE 8080

# Environment defaults
ENV APP_ENV=production \
    PORT=8080 \
    HOST=0.0.0.0 \
    RUST_LOG=info

# Set entry point
ENTRYPOINT ["/usr/local/bin/edge-api"]
