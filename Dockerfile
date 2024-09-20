FROM rust:alpine AS builder
WORKDIR /usr/src/quaver-bio
COPY . .
RUN cargo build --release

FROM alpine:latest
COPY --from=builder /usr/src/quaver-bio/target/release/quaver-bio /usr/local/bin/quaver-bio
ENTRYPOINT ["/usr/local/bin/quaver-bio"]
