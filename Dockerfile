# --- Stage 1: Build the application ---
# We use the official Rust image as a build environment.
# Using a specific version ensures repeatable builds.
FROM rust:1.78 as builder

# Install the C toolchain (linker) required for static MUSL builds.
RUN apt-get update && apt-get install -y musl-tools

# Install the Rust target for musl.
RUN rustup target add x86_64-unknown-linux-musl

# Create a new, empty workspace in the build environment.
WORKDIR /usr/src/app

# Create a Cargo configuration file to explicitly use the musl-gcc linker.
RUN mkdir .cargo
RUN echo '[target.x86_64-unknown-linux-musl]\nlinker = "musl-gcc"' > .cargo/config.toml

# Copy the Cargo manifest files to leverage Docker layer caching.
COPY Cargo.toml Cargo.lock ./

# Copy your actual source code.
COPY src ./src

# Copy the static game data into the build environment.
COPY data ./data

# Build the application in release mode for performance.
RUN cargo build --release --target x86_64-unknown-linux-musl


# --- Stage 2: Create the final, small runtime image ---
# We use a "distroless" image for a minimal and secure final container.
FROM gcr.io/distroless/static-debian12

# Copy the compiled binary from the builder stage to the /server (absolute path).
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/server /server

# Copy the data directory to /data (absolute path).
COPY --from=builder /usr/src/app/data /data

# Set the environment variable for the data directory.
# This tells our application where to find the game data inside the container.
ENV DATA_DIR=/data

# Set the entrypoint to the absolute path of our application binary.
ENTRYPOINT ["/server"]
