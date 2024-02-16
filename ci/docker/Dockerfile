FROM cern/cc7-base

# This repo file references a URL that is no longer valid. It also isn't used by the build
# toolchain, so we can safely remove it entirely
RUN rm /etc/yum.repos.d/epel.repo

# https://linux.web.cern.ch/centos7/docs/softwarecollections/#inst
# Tools needed for the build and setup process
RUN yum -y install wget tar
# Fetch the repo information for the devtoolset repo
RUN yum install -y centos-release-scl
# Install more recent GCC and binutils, to allow us to compile
RUN yum install -y devtoolset-8

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
