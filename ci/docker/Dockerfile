FROM centos:6.10

# CentOS 6 packages are no longer hosted on the main repository, instead they are in the CentOS Vault
RUN sed -i 's/^mirrorlist/#mirrorlist/g' /etc/yum.repos.d/CentOS-Base.repo && \
    sed -i 's/#baseurl=http:\/\/mirror.centos.org\/centos\/$releasever/baseurl=http:\/\/vault.centos.org\/6.10/g' /etc/yum.repos.d/CentOS-Base.repo

# Set up additional build tools
RUN yum -y update && yum clean all

# TODO: build latest version of GCC from source
