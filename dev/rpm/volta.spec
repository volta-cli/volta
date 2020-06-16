Name:           volta
Version:        0.8.2
Release:        1%{?dist}
Summary:        The JavaScript Launcher ⚡

License:        BSD 2-CLAUSE
URL:            https://%{name}.sh
Source0:        https://github.com/volta-cli/volta/archive/v%{version}.tar.gz

# cargo is required, but installing from RPM is failing with libcrypto dep error
# so you will have to install cargo manually to build this
#BuildRequires:  cargo

# because these are built with openssl
Requires:       openssl


%description
Volta’s job is to manage your JavaScript command-line tools, such as node, npm, yarn, or executables shipped as part of JavaScript packages. Similar to package managers, Volta keeps track of which project (if any) you’re working on based on your current directory. The tools in your Volta toolchain automatically detect when you’re in a project that’s using a particular version of the tools, and take care of routing to the right version of the tools for you.


%prep
# this unpacks the tarball to the build root
%setup -q


%build
# build the release binaries
# NOTE: build expects to `cd` into a volta-<version> directory
cargo build --release


# this installs into a chroot directory resembling the user's root directory
%install
# BUILDROOT/usr/bin
%define volta_install_dir %{buildroot}/%{_bindir}
# setup the /usr/bin/volta-lib/ directory
rm -rf %{buildroot}
mkdir -p %{volta_install_dir}
# install everything into into /usr/bin/, so it's on the PATH
install -m 0755 target/release/%{name} %{volta_install_dir}/%{name}
install -m 0755 target/release/volta-shim %{volta_install_dir}/volta-shim
install -m 0755 target/release/volta-migrate %{volta_install_dir}/volta-migrate


# files installed by this package
%files
%license LICENSE
%{_bindir}/%{name}
%{_bindir}/volta-shim
%{_bindir}/volta-migrate


# this runs before install
%pre
# make sure the /usr/bin/volta/ dir does not exist, from prev RPM installs (or this will fail)
printf '\033[1;32m%12s\033[0m %s\n' "Running" "Volta pre-install..." 1>&2
rm -rf %{_bindir}/%{name}


# this runs after install, and sets up VOLTA_HOME and the shell integration
%post
printf '\033[1;32m%12s\033[0m %s\n' "Running" "Volta post-install setup..." 1>&2
# run this as the user who invoked sudo (not as root, because we're writing to $HOME)
/bin/su -c "%{_bindir}/volta setup" - $SUDO_USER


%changelog
* Tue Oct 22 2019 Charles Pierce <cpierce.grad@gmail.com> - 0.6.5-1
- Update to use 'volta setup' as the postinstall script
* Mon Jun 03 2019 Michael Stewart <mikrostew@gmail.com> - 0.5.3-1
- First volta package
