FROM rust:1.31-nightly-slim

WORKDIR /usr/src/nightly
COPY . .

RUN cargo install --path .

CMD ["nightly"]