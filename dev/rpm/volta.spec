Name:           volta
Version:        0.5.3
Release:        1%{?dist}
Summary:        The JavaScript Launcher ⚡

License:        BSD 2-CLAUSE
URL:            https://%{name}.sh
Source0:        https://github.com/volta-cli/volta/archive/v%{version}.tar.gz

# cargo is required, but installing from RPM is failing with libcrypto dep error
# so you will have to install cargo manually
#BuildRequires:  cargo

# TODO - should require openssl?
Requires:       bash

# TODO
#BuildArch:

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
# setup the /usr/bin/volta/ directory
rm -rf %{buildroot}
mkdir -p %{buildroot}/%{_bindir}/%{name}
# install the compiled binaries into /usr/bin/volta/
install -m 0755 target/release/%{name} %{buildroot}/%{_bindir}/%{name}/%{name}
install -m 0755 target/release/shim %{buildroot}/%{_bindir}/%{name}/shim
# and put the postinstall script there too
install -m 0755 dev/rpm/volta-postinstall.sh %{buildroot}/%{_bindir}/%{name}/volta-postinstall.sh


# files installed by this package
%files
%license LICENSE
%{_bindir}/%{name}/%{name}
%{_bindir}/%{name}/shim
%{_bindir}/%{name}/volta-postinstall.sh


# this runs after install, and sets up VOLTA_HOME and the shell integration
%post
echo "Running Volta post-install setup..."
%{_bindir}/%{name}/volta-postinstall.sh


%changelog
* Mon Jun 03 2019 Michael Stewart <mikrostew@gmail.com> - 0.5.3-1
- First volta package
