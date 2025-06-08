FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev git
WORKDIR /mnt/source
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache git

COPY --from=builder /mnt/source/target/release/clean /opt/io.github.black-desk/clean/bin/clean

VOLUME ["/mnt"]
RUN git config --global --add safe.directory /mnt
WORKDIR /mnt

ENTRYPOINT ["/opt/io.github.black-desk/clean/bin/clean"]
