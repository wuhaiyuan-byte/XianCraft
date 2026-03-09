# This is a simplified Dockerfile used ONLY for packaging within cloudbuild.yaml
# It does not perform any builds.

# --- Use the minimal static image from Google ---
FROM gcr.io/distroless/static-debian12

# --- Copy the pre-built files ---
# The working directory is /workspace in Google Cloud Build

# Copy the static backend binary built in the 'Build Backend' step
COPY --chown=nonroot:nonroot target/x86_64-unknown-linux-musl/release/server /server

# Copy the static frontend assets built in the 'Build Frontend' step
COPY --chown=nonroot:nonroot client/dist /dist

# Copy the data directory
COPY --chown=nonroot:nonroot data /data

# --- Set environment variables and entrypoint ---
ENV DATA_DIR=/data
ENV STATIC_DIR=/dist

ENTRYPOINT ["/server"]
