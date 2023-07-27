# See https://hub.docker.com/_/rust/

FROM rust:alpine as builder
WORKDIR /usr/src/near-facsimile
COPY . .
RUN apk update
RUN apk add musl-dev
RUN cargo install --path .

FROM alpine:latest
COPY --from=builder /usr/local/cargo/bin/near-facsimile /usr/local/bin/near-facsimile
# When running this container interactively, use `-v .:/mnt:Z`
# to mount the current directory in the host to the container working dir.
VOLUME ["/mnt"]
WORKDIR "/mnt"
CMD ["near-facsimile"]
