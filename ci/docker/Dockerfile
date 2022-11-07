FROM cern/slc6-base

# This repo file references a URL that is no longer valid. It also isn't used by the build
# toolchain, so we can safely remove it entirely
RUN rm /etc/yum.repos.d/epel.repo

# Tools needed for the build and setup process
RUN yum -y install wget tar
# Fetch the repo information for the devtoolset repo
RUN wget -O /etc/yum.repos.d/slc6-devtoolset.repo http://linuxsoft.cern.ch/cern/devtoolset/slc6-devtoolset.repo
# Install more recent GCC and binutils, to allow us to compile 
RUN yum -y install devtoolset-2-gcc devtoolset-2-binutils

# Install Rust toolchain
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.63
ENV PATH="/root/.cargo/bin:${PATH}"
