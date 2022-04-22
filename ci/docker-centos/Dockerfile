FROM centos:6.10

# CentOS 6 packages are no longer hosted on the main repository, instead they are in the CentOS Vault
RUN sed -i 's/^mirrorlist/#mirrorlist/g' /etc/yum.repos.d/CentOS-Base.repo && \
    sed -i 's/#baseurl=http:\/\/mirror.centos.org\/centos\/$releasever/baseurl=http:\/\/linuxsoft.cern.ch\/centos-vault\/6.10/g' /etc/yum.repos.d/CentOS-Base.repo

# Set up additional build tools
RUN yum -y update && yum clean all
RUN yum -y install gcc curl openssl openssl-devel ca-certificates tar perl perl-Module-Load-Conditional && yum clean all

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"
