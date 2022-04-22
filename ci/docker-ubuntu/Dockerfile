FROM ubuntu:22.04

# Install build tools
RUN apt-get update -y; \
    apt-get install -y curl build-essential pkg-config libssl-dev

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"
