# =============================================================================
# organiza Dockerfile
# Multi-stage build for minimal, secure production image
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Build environment
# rust:1-alpine provides musl libc by default (statically linked binary)
# -----------------------------------------------------------------------------
FROM rust:1-alpine AS builder

# Build arguments for flexibility
ARG TARGETPLATFORM
ARG BUILDPLATFORM

WORKDIR /build

# Install build dependencies
# - musl-dev: Required for musl libc linking
# Pure-Rust dependencies: pkgconf is NOT needed (no C/system libs)
RUN apk add --no-cache \
    musl-dev

# Copy only dependency files first for better layer caching
# This means dependencies are only rebuilt when Cargo.toml/Cargo.lock change
COPY Cargo.toml Cargo.lock ./

# Create a dummy project to build dependencies
# This exploits Docker's layer caching - dependencies change less often than source
RUN mkdir src && \
    echo 'fn main() { println!("dummy"); }' > src/main.rs && \
    echo 'pub fn dummy() {}' > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY src ./src

# Touch main.rs to invalidate the dummy build, then build the real binary
# The dependencies are already cached from the previous step
# --locked keeps the build reproducible against Cargo.lock
RUN touch src/main.rs src/lib.rs && \
    cargo build --release --locked

# Verify the binary was built and strip debug symbols
RUN strip /build/target/release/organiza && \
    /build/target/release/organiza --version

# -----------------------------------------------------------------------------
# Stage 2: Runtime environment
# Minimal Alpine image - no Rust toolchain, no build dependencies
# -----------------------------------------------------------------------------
FROM alpine:3.23 AS runtime

# Labels for container metadata (OCI standard)
LABEL org.opencontainers.image.title="organiza" \
      org.opencontainers.image.description="Cross-platform configurable file organizer" \
      org.opencontainers.image.url="https://github.com/dallay/file-organizer" \
      org.opencontainers.image.source="https://github.com/dallay/file-organizer" \
      org.opencontainers.image.vendor="Yuniel Acosta" \
      org.opencontainers.image.licenses="MIT"

# Install runtime dependencies
# - ca-certificates: For HTTPS connections (future-proofing)
# - tini: Proper init system for containers (PID 1 signal handling)
RUN apk add --no-cache \
    ca-certificates \
    tini

# Create non-root user for security
# Running as root in containers is a security anti-pattern
RUN addgroup -g 1000 organiza && \
    adduser -u 1000 -G organiza -s /bin/sh -D organiza

# Copy binary from builder stage
COPY --from=builder /build/target/release/organiza /usr/local/bin/organiza

# Ensure binary is executable
RUN chmod +x /usr/local/bin/organiza

# Switch to non-root user
USER organiza

# Set working directory for mounted volumes
WORKDIR /workspace

# Use tini as entrypoint for proper signal handling
# This ensures SIGTERM is properly forwarded to the process
ENTRYPOINT ["/sbin/tini", "--", "organiza"]

# Default command shows help
CMD ["--help"]
