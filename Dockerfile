# Build stage
FROM rust:1.86-alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache musl-dev

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release --locked

# Runtime stage
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy binary from builder
COPY --from=builder /app/target/release/null-e /usr/local/bin/null-e

# Create non-root user
RUN adduser -D -u 1000 nulle
USER nulle

WORKDIR /workspace

ENTRYPOINT ["null-e"]
CMD ["--help"]
