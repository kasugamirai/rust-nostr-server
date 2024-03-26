# Nostr Relay Project (Work in Progress)

This project is based on the [rust-nostr](https://github.com/rust-nostr/nostr) implementation and serves as a Nostr
relay.

## Prerequisites

Ensure that you have **Docker**, **docker-compose**, and necessary script execution permissions on your system. If you
intend to build and run without Docker, you also need the **sh** shell and the project's build dependencies installed.

## Getting Started

Here's how you can set up and run the project:

### Running Without Docker

1. **Build the project**: Use the provided shell script for building the project.

    ```bash
    sh build.sh
    ```

2. **Navigate to the output directory**: The build output directory contains the compiled files and scripts.

    ```bash
    cd output
    ```

3. **Run the script**: Execute the generated script to run the project.

    ```bash
    sh script.sh
    ```

### Running With Docker

1. **Build the Docker image**: This uses the `docker-compose.yml` configuration to create Docker image for the service.

    ```bash
    docker-compose build
    ```

2. **Start the service**: This command starts the Docker service for the Nostr relay.

    ```bash
    docker-compose up
    ```