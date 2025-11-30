# Multi-stage Dockerfile for Typf builds

# Stage 1: Builder
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libfreetype6-dev \
    libfontconfig1-dev \
    libharfbuzz-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY backends/ backends/
COPY bindings/ bindings/

# Build the project
RUN cargo build --workspace --release

# Run tests
RUN cargo test --workspace --release

# Stage 2: Minimal runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libfreetype6 \
    libfontconfig1 \
    libharfbuzz0b \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binaries
COPY --from=builder /build/target/release/typf-cli /usr/local/bin/typf

# Create a non-root user
RUN useradd -m -u 1000 typf && \
    mkdir -p /data && \
    chown typf:typf /data

USER typf
WORKDIR /data

ENTRYPOINT ["typf"]
CMD ["--help"]

# Metadata
LABEL org.opencontainers.image.title="Typf"
LABEL org.opencontainers.image.description="Modular text rendering pipeline"
LABEL org.opencontainers.image.version="2.0.0-dev"
LABEL org.opencontainers.image.authors="FontLab"
