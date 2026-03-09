# --- Stage 1: Build the application ---
# We use the official Rust image as a build environment.
# Using a specific version ensures repeatable builds.
FROM rust:1.78 as builder

# Create a new, empty workspace in the build environment.
WORKDIR /usr/src/app

# Install the musl target, which is needed for creating a static binary
RUN rustup target add x86_64-unknown-linux-musl

# Copy the Cargo manifest files. This is done separately to leverage Docker's layer caching.
# If these files don't change, Docker won't re-download dependencies.
COPY Cargo.toml Cargo.lock ./

# Copy your actual source code.
COPY src ./src

# Build the application in release mode for performance.
# This creates a statically linked binary in /usr/src/app/target/release/
RUN cargo build --release --target x86_64-unknown-linux-musl


# --- Stage 2: Create the final, small runtime image ---
# We use a "distroless" image which is a barebones image containing only the essentials.
# This significantly reduces the image size and improves security.
FROM gcr.io/distroless/cc-static

# Copy the compiled binary from the builder stage to the final image.
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/xiancraft /

# Set the entrypoint of the container to our application binary.
# The server will start on port 8080 by default, which is what Cloud Run expects.
ENTRYPOINT ["/xiancraft"]
