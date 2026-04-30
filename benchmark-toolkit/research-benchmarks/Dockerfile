# Benchmark Runner Dockerfile
# Purpose: Automated benchmark orchestrator comparing ProvChain-Org vs Neo4j
# Language: Rust for performance and type safety

FROM rust:1.85-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy actual source code and build directly.
# The benchmark-runner crate is both a library and binary crate; avoiding the
# dummy warmup build prevents stale package artifacts from masking source changes.
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the actual binary
RUN cargo build --release --locked

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /benchmark

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies in an isolated virtualenv
RUN python3 -m venv /opt/benchmark-venv
ENV PATH="/opt/benchmark-venv/bin:${PATH}"
RUN pip install --upgrade pip && pip install pandas matplotlib seaborn requests rdflib

# Copy the binary from builder
COPY --from=builder /build/target/release/benchmark-runner /benchmark/

# Copy datasets and scripts
COPY datasets /benchmark/datasets
COPY scripts /benchmark/scripts

# Create results directory
RUN mkdir -p /benchmark/results

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Default command - run all benchmarks
CMD ["/benchmark/benchmark-runner", "--all"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1
