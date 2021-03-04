# Build Stage
FROM rustlang/rust:nightly-alpine as builder
LABEL maintainer="jakearmendariz <jakearmendariz99@gmail.com>"
LABEL version="0.1.0"

WORKDIR /code
RUN apk update \
    && apk add build-base openssl-dev zlib-dev  \
    && rm -rf /var/cache/apk/*
COPY . .
RUN cargo build

# Image Stage
FROM alpine:latest
LABEL maintainer="jakearmendariz <jakearmendariz99@gmail.com>"
LABEL version="0.1.0"

ENV ROCKET_ENV=development \
    ROCKET_ADDRESS=0.0.0.0 ROCKET_PORT=8000 \
    ROCKET_LOG=critical \
    VIEW="10.10.0.4:13800,10.10.0.5:13800" \
    REPL_FACTOR=1

EXPOSE 8000

COPY --from=builder /code/target/debug/loadbalancer /usr/local/bin/loadbalancer
CMD loadbalancer

# [docker]
# name = "loadbalancer"
# version = "0.1.0"
# maintainer = "jakearmendariz <jakearmendariz99@gmail.com>"
# temp_folder = "./.tmp_docker"
# tag = "loadbalancer"

# [development]
# address = "0.0.0.0"
# port = 8000

# [production]
# address = "0.0.0.0"
# port = 8000