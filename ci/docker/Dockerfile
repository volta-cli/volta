FROM ubuntu:18.04

# FROM centos:6.10
# # CentOS 6 packages are no longer hosted on the main repository, instead they are in the CentOS Vault
# RUN sed -i 's/^mirrorlist/#mirrorlist/g' /etc/yum.repos.d/CentOS-Base.repo && \
#     sed -i 's/#baseurl=http:\/\/mirror.centos.org\/centos\/$releasever/baseurl=http:\/\/linuxsoft.cern.ch\/centos-vault\/6.10/g' /etc/yum.repos.d/CentOS-Base.repo

# # Set up additional build tools
# RUN yum -y update && yum clean all
# RUN yum -y install curl bash gcc gcc-c++ svn texinfo-tex flex zip wget && yum clean all

RUN apt-get update -y

ARG TARGET
COPY ./ci/ ./ci/
RUN chmod +x ./ci/install-build-tools.sh && bash ./ci/install-build-tools.sh $TARGET

# Install Rust
RUN apt-get install -y curl bash
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV CARGO_HOME=/root/workspace/.cargo \
    PATH="/root/.cargo/bin:${PATH}"
RUN rustup target add $TARGET