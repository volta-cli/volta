Name:           volta
Version:        0.6.3
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
# /usr/bin/volta-lib/
%define volta_bin_dir %{_bindir}/%{name}-lib
# BUILDROOT/usr/bin/volta-lib
%define volta_install_dir %{buildroot}/%{volta_bin_dir}
# setup the /usr/bin/volta-lib/ directory
rm -rf %{buildroot}
mkdir -p %{volta_install_dir}
# install the `volta` binary into /usr/bin/, so it's on the PATH
install -m 0755 target/release/%{name} %{buildroot}/%{_bindir}/%{name}
# install everything else to /usr/bin/volta-lib/ (so they are not on the PATH)
# the `shim` binary
install -m 0755 target/release/shim %{volta_install_dir}/shim
# the postinstall script
install -m 0755 dev/rpm/volta-postinstall.sh %{volta_install_dir}/volta-postinstall.sh
# the shell integration scripts
# these are loaded for the user that installed the RPM
install -m 0644 shell/unix/load.sh %{volta_install_dir}/load.sh
install -m 0644 shell/unix/load.fish %{volta_install_dir}/load.fish


# files installed by this package
%files
%license LICENSE
%{_bindir}/%{name}
%{volta_bin_dir}/shim
%{volta_bin_dir}/volta-postinstall.sh
%{volta_bin_dir}/load.sh
%{volta_bin_dir}/load.fish


# this runs before install
%pre
# make sure the /usr/bin/volta/ dir does not exist, from prev RPM installs (or this will fail)
printf '\033[1;32m%12s\033[0m %s\n' "Running" "Volta pre-install..." 1>&2
rm -rf %{_bindir}/%{name}


# this runs after install, and sets up VOLTA_HOME and the shell integration
%post
printf '\033[1;32m%12s\033[0m %s\n' "Running" "Volta post-install setup..." 1>&2
# run this as the user who invoked sudo (not as root, because we're writing to $HOME)
/bin/su -c %{volta_bin_dir}/volta-postinstall.sh - $SUDO_USER


# this is called after package uninstall _and_ upgrade, but we only want to remove these for uninstall
# - it is passed "the number of packages that will be left after this step is completed",
#   so it checks that value - 0 means uninstall, anything else is upgrade
%postun
# only run these for uninstall
if [ $1 -eq 0 ]; then
  printf '\033[1;32m%12s\033[0m %s\n' "Removing" "~/.volta/ directory" 1>&2
  # run this as the user who invoked sudo (not as root, because we're using $HOME)
  # and using single quotes so $HOME doesn't expand here (for root), but expands in the user's shell
  /bin/su -c 'rm -rf $HOME/.volta' - $SUDO_USER
  # the RPM removes the binaries in this dir, but not the dir itself
  printf '\033[1;32m%12s\033[0m %s\n' "Removing" %{volta_bin_dir}" directory" 1>&2
  rm -rf %{volta_bin_dir}
fi


%changelog
* Mon Jun 03 2019 Michael Stewart <mikrostew@gmail.com> - 0.5.3-1
- First volta package
