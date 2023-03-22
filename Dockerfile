# Use the official Rust image as the base image
FROM rust:1.67.0 as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files to the working directory
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs file to build dependencies
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies only to cache them
RUN cargo build --release

# Remove the dummy main.rs file
RUN rm src/main.rs

# Copy the rest of the source code
COPY src ./src

# Rebuild the application with the actual source code
RUN cargo build --release

# Create a new lightweight image for the final build
FROM debian:buster-slim

# Install SSL certificates
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/local/bin

# Copy the binary from the builder image
COPY --from=builder /usr/src/app/target/release/discord_gpt .

# Set the entrypoint to run the binary
CMD ["./discord_gpt"]