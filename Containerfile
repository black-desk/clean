FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev git
WORKDIR /mnt/source
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache git
RUN git config --global --add safe.directory /mnt

COPY --from=builder /mnt/source/target/release/clean /opt/io.github.black-desk/clean/bin/clean

WORKDIR /mnt
VOLUME ["/mnt"]

ENTRYPOINT ["/opt/io.github.black-desk/clean/bin/clean"]
