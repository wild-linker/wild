ARG BASE_IMAGE=ghcr.io/wild-linker/wild/ci-ubuntu-amd64:latest
FROM ${BASE_IMAGE}

RUN apt-get update \
    && eatmydata apt-get -y install clang-22 lld-22 llvm-22-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN eatmydata rustup toolchain install nightly \
        --allow-downgrade \
        --target $(uname -m)-unknown-linux-musl \
        --component rustc-codegen-cranelift-preview
