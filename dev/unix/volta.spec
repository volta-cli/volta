Name:           volta
Version:        0.5.3
Release:        1%{?dist}
Summary:        The JavaScript Launcher ⚡

License:        BSD 2-CLAUSE
URL:            https://volta.sh
Source0:        https://github.com/volta-cli/volta/archive/v%{version}.tar.gz

BuildRequires:
Requires:

%description
Volta’s job is to manage your JavaScript command-line tools, such as node, npm, yarn, or executables shipped as part of JavaScript packages. Similar to package managers, Volta keeps track of which project (if any) you’re working on based on your current directory. The tools in your Volta toolchain automatically detect when you’re in a project that’s using a particular version of the tools, and take care of routing to the right version of the tools for you.


%prep
%setup -q


%build
TODO


%install
TODO
rm -rf $RPM_BUILD_ROOT
%make_install


%files
%doc
TODO



%changelog
* Thu May 30 2019 Michael Stewart <mikrostew@gmail.com> - 0.5.3-1
- First volta package
