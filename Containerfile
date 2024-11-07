# Use the official Rust image as the build stage
FROM rust:1-alpine3.20 as builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files to the working directory
COPY . /usr/src/app/

# Pre-build dependencies to cache them
RUN cargo build --release

# Use a smaller base image for the final stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache libgcc libstdc++ openssl

# Set the working directory
WORKDIR /usr/src/app

# Copy the compiled binary from the build stage
COPY --from=builder /usr/src/app/target/release/website4share .

# Expose the application port
EXPOSE 8080

# Run the application
CMD ["./website4share"]