FROM ubuntu:20.04

ARG DEBIAN_FRONTEND=noninteractive
ARG RUST_TOOLCHAIN=stable

RUN apt update \
    && apt install -y \
        build-essential \
        curl

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH=/root/.cargo/bin:$PATH

RUN rustup update \
    && rustup toolchain install $RUST_TOOLCHAIN \
    && rustup default $RUST_TOOLCHAIN \
    && rustup target add wasm32-wasi --toolchain $RUST_TOOLCHAIN
