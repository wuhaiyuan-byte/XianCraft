# --- Stage 1: Build the frontend ---
FROM node:20 as frontend_builder
WORKDIR /usr/src/client
COPY client/package.json client/package-lock.json ./
RUN npm install
COPY client/ ./
RUN npm run build

# --- Stage 2: Build the application ---
FROM rust:1.78 as builder
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/app
RUN mkdir .cargo
RUN echo '[target.x86_64-unknown-linux-musl]\nlinker = "musl-gcc"' > .cargo/config.toml
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY data ./data
RUN cargo build --release --target x86_64-unknown-linux-musl

# --- DEBUG: Check for dynamic dependencies ---
RUN ldd /usr/src/app/target/x86_64-unknown-linux-musl/release/server

# --- Stage 3: Create a temporary debugging image ---
FROM debian:bookworm-slim

COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/server /server
COPY --from=builder /usr/src/app/data /data
COPY --from=frontend_builder /usr/src/client/dist /dist

# Set environment variables for the application
ENV DATA_DIR=/data
ENV STATIC_DIR=/dist
# This ENV is for local running; Cloud Run will provide its own PORT.
ENV PORT=8080

# Debug: List files and permissions at the root, then attempt to start the server.
# This will show us if the /server file exists and if it's executable.
ENTRYPOINT ["/bin/sh", "-c", "echo '--- Listing root directory ---' && ls -l / && echo '--- Attempting to start server ---' && /server"]
