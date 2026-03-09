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

# --- Stage 3: Create the final, small runtime image ---
FROM gcr.io/distroless/static-debian12

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/server /server

# Copy the data directory
COPY --from=builder /usr/src/app/data /data

# Copy the built frontend from the frontend_builder stage
COPY --from=frontend_builder /usr/src/client/dist /dist

# Set environment variables for the application
ENV DATA_DIR=/data
ENV STATIC_DIR=/dist

ENTRYPOINT ["/server"]
