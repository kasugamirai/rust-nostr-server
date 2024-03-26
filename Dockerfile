# Stage 1: Build the application
FROM rust:1.77 as builder
WORKDIR /usr/src/myapp
COPY . .
RUN apt-get update && apt-get install -y clang libclang-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

# Stage 2: Create the runtime environment
FROM rust:1.77
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the compiled application from the builder stage
COPY --from=builder /usr/src/myapp/target/release/bootstrap /usr/local/bin/bootstrap

# Set the default command to run the application
CMD ["bootstrap"]