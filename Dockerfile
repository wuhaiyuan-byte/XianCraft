# --- Stage 1: Build the application ---
# We use the official Rust image as a build environment.
# Using a specific version ensures repeatable builds.
FROM rust:1.78 as builder

# Install the C toolchain (linker) required for static MUSL builds.
# This is the key fix that was missing before.
RUN apt-get update && apt-get install -y musl-tools

# Install the Rust target for musl.
RUN rustup target add x86_64-unknown-linux-musl

# Create a new, empty workspace in the build environment.
WORKDIR /usr/src/app

# Create a Cargo configuration file to explicitly use the musl-gcc linker.
# This ensures Cargo uses the tools we just installed.
RUN mkdir .cargo
RUN echo '[target.x86_64-unknown-linux-musl]\nlinker = "musl-gcc"' > .cargo/config.toml

# Copy the Cargo manifest files to leverage Docker layer caching.
COPY Cargo.toml Cargo.lock ./

# Copy your actual source code.
COPY src ./src

# Build the application in release mode for performance.
# This will now succeed because the linker is present and configured.
RUN cargo build --release --target x86_64-unknown-linux-musl


# --- Stage 2: Create the final, small runtime image ---
# We use a "distroless" image for a minimal and secure final container.
FROM gcr.io/distroless/static-debian12

# Copy the compiled binary from the builder stage to the final image.
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/xiancraft /

# Set the entrypoint of the container to our application binary.
# Cloud Run will automatically use port 8080.
ENTRYPOINT ["/xiancraft"]
