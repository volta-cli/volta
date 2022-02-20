FROM rust:alpine
COPY . ./volta
RUN ls /volta
RUN apk --print-arch
RUN rustup show active-toolchain
RUN cd volta && cargo build --release
RUN /volta/target/release/volta --version
