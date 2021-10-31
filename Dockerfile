FROM rust:1.54.0 as builder

WORKDIR .

# Update and install cross-platform build dependencies
RUN apt-get update
RUN apt-get install -y \
    gcc \
    g++ \
    gcc-multilib \
    g++-multilib \
    build-essential \
    xutils-dev \
    libsdl2-dev \
    libsdl2-gfx-dev \
    libsdl2-image-dev \
    libsdl2-mixer-dev \
    libsdl2-net-dev \
    libsdl2-ttf-dev \
    libreadline6-dev \
    libncurses5-dev \
    mingw-w64 \
    cmake

COPY ./ .
RUN cargo install --path .

RUN cargo test

RUN cargo build --release
