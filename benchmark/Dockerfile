# Benchmark Runner Dockerfile
# Purpose: Automated benchmark orchestrator comparing ProvChain-Org vs Neo4j
# Language: Rust for performance and type safety

FROM rust:1.75-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo.toml first for better caching
COPY Cargo.toml ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual binary
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /benchmark

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Python for visualization scripts
RUN pip3 install pandas matplotlib seaborn requests

# Copy the binary from builder
COPY --from=builder /build/release/benchmark-runner /benchmark/

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
