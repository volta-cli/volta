FROM centos:6

# Set up additional build tools
RUN yum -y update && yum clean all
RUN yum -y install gcc curl openssl openssl-devel ca-certificates tar perl perl-Module-Load-Conditional && yum clean all

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"
