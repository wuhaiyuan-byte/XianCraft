# --- Stage 1: Build the frontend ---
FROM node:20 as frontend_builder
WORKDIR /usr/src/client
COPY client/package.json client/package-lock.json ./
RUN npm install
COPY client/ ./
RUN npm run build

# --- Stage 2: Build the application with dependency caching ---
# This is the optimized build stage for a Rust project.
FROM ekidd/rust-musl-builder:latest as builder

WORKDIR /home/rust/src

# Step 2a: Build dependencies first to leverage Docker layer caching.
# Copy only the manifests.
COPY --chown=rust:rust Cargo.toml Cargo.lock ./

# Step 2b: Create a dummy main.rs to allow a successful dependency-only build.
RUN mkdir -p src && echo "fn main() {println!(\"Building dependencies...\");}" > src/main.rs

# Step 2c: Build the dummy project. This compiles and caches all dependencies.
# This layer will only be re-run if Cargo.toml or Cargo.lock change.
RUN cargo build --release

# Step 2d: Copy the actual application source code, overwriting the dummy main.rs.
COPY --chown=rust:rust ./src ./src

# Step 2e: Build the actual application. This will be much faster as all
# dependencies are already compiled and cached.
RUN cargo build --release

# --- Stage 3: Create the final, small runtime image ---
FROM gcr.io/distroless/static-debian12

# Copy the truly static binary from the builder.
COPY --chown=nonroot:nonroot --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/server /server

# Copy the necessary data and static web assets, setting correct ownership.
COPY --chown=nonroot:nonroot --from=builder /home/rust/src/data /data
COPY --chown=nonroot:nonroot --from=frontend_builder /usr/src/client/dist /dist

# Set environment variables for the application.
ENV DATA_DIR=/data
ENV STATIC_DIR=/dist

# The entrypoint is now a guaranteed-to-be-static executable.
ENTRYPOINT ["/server"]
