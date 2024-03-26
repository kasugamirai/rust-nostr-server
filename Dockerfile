# Use the official Rust image as the base image
FROM rust:1.77 as builder

# Set the working directory in the Docker container
WORKDIR /usr/src/myapp

# Copy the source code into the container
COPY . .

# Compile the application in release mode
RUN cargo build --release

# Use the Debian Slim image for the runtime environment
FROM debian:buster-slim

# Install required packages and remove the cache to keep the image small
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the compiled application from the builder stage
COPY --from=builder /usr/src/myapp/target/release/bootstrap /usr/local/bin/bootstrap

# Set the default command to run the application
CMD ["bootstrap"]
