# Use a rust alpine image as the builder image
FROM rust:alpine AS builder
WORKDIR /usr/src/quaver-bio
# Copy the source code into the container
COPY . .
# Install the required dependencies
RUN apk add --no-cache musl-dev libressl-dev
# Build the binary in release mode
RUN cargo build --release

# Use the alpine image as the final image
FROM alpine:latest
# Copy the binary from the builder image into the final image
COPY --from=builder --chown=900:900 /usr/src/quaver-bio/target/release/quaver-bio /usr/local/bin/quaver-bio
# Run the binary as a non-root user
USER 900:900
ENTRYPOINT ["/usr/local/bin/quaver-bio"]
