FROM rust:1.54.0

WORKDIR .

COPY ./ .

RUN cargo test

