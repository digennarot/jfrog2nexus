# Builder stage
FROM rust:1.93-slim-trixie AS builder

# Install build dependencies required by crates (like reqwest/sqlx needing openssl via rustls-tls/native-tls fallback or similar)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy the entire project
COPY . .

# Build the application
RUN cargo build --release

# Final runtime image
FROM debian:trixie-slim

# Install runtime dependencies required for HTTPS requests (reqwest) and metrics / db connections
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/jfrog2nexus /usr/local/bin/jfrog2nexus

# Set the entrypoint
ENTRYPOINT ["jfrog2nexus"]
CMD ["--help"]
