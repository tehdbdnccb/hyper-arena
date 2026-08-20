# Stage 1: Build environment
FROM rustlang/rust:nightly AS builder
WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests and source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Build for release
# SQLx needs a dummy DATABASE_URL during build to validate queries
# The actual DB connection is made at runtime
RUN SQLX_OFFLINE=true cargo build --release

# Stage 2: Production runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/backend /app/backend

# Expose the WebSocket port
EXPOSE 4000

# Run the binary
CMD ["./backend"]

