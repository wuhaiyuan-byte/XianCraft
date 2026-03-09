# --- Stage 1: Build the frontend ---
FROM node:20 as frontend_builder
WORKDIR /usr/src/client
COPY client/package.json client/package-lock.json ./
RUN npm install
COPY client/ ./
RUN npm run build

# --- Stage 2: Build the application using a dedicated static MUSL builder ---
# This image is purpose-built to create fully static Rust binaries.
FROM ekidd/rust-musl-builder:latest as builder

# The source code is copied to /home/rust/src, which is the standard for this image.
COPY --chown=rust:rust . .

# First, update the crate index and dependencies to ensure we can find recent versions.
# This is the fix for the axum version resolution failure.
RUN cargo update

# Now, run the build. The toolchain in this image is pre-configured for static linking.
# No --target flag is needed.
RUN cargo build --release

# --- Stage 3: Create the final, small runtime image ---
FROM gcr.io/distroless/static-debian12

# Copy the truly static binary from the builder.
# The output path is different due to the builder image's structure.
COPY --chown=nonroot:nonroot --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/server /server

# Copy the necessary data and static web assets, setting correct ownership.
COPY --chown=nonroot:nonroot --from=builder /home/rust/src/data /data
COPY --chown=nonroot:nonroot --from=frontend_builder /usr/src/client/dist /dist

# Set environment variables for the application.
ENV DATA_DIR=/data
ENV STATIC_DIR=/dist

# The entrypoint is now a guaranteed-to-be-static executable.
ENTRYPOINT ["/server"]
