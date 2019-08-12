#!/usr/bin/env bash
# Build an RPM package for Volta

# using the directions from https://rpm-packaging-guide.github.io/

# exit on error
set -e

# only argument is the version number
release_version="${1:?Must specify the release version, like \`build-rpm 1.2.3\`}"
archive_filename="v${release_version}.tar.gz"

# make sure these packages are installed
# (https://rpm-packaging-guide.github.io/#prerequisites)
sudo yum install gcc rpm-build rpm-devel rpmlint make python bash coreutils diffutils patch rpmdevtools

# set up the directory layout for the RPM packaging workspace
# (https://rpm-packaging-guide.github.io/#rpm-packaging-workspace)
rpmdev-setuptree

# create a tarball of the repo for the specified version
# using prefix because the rpmbuild process expects a 'volta-<version>' directory
# (https://rpm-packaging-guide.github.io/#putting-source-code-into-tarball)
git archive --format=tar.gz --output=$archive_filename --prefix="volta-${release_version}/" HEAD

# move the archive to the SOURCES dir, after cleaning it up
# (https://rpm-packaging-guide.github.io/#working-with-spec-files)
rm -rf "$HOME/rmpbuild/SOURCES/"*
mv "$archive_filename" "$HOME/rpmbuild/SOURCES/"

# copy the .spec file to SPECS dir
cp dev/rpm/volta.spec "$HOME/rpmbuild/SPECS/"

# build it!
# (https://rpm-packaging-guide.github.io/#binary-rpms)
rpmbuild -bb "$HOME/rpmbuild/SPECS/volta.spec"
# (there will be a lot of output)

# then install it and verify everything worked...
echo ""
echo "Build finished!"
echo ""
echo "Run this to install:"
echo "  \`sudo yum install ~/rpmbuild/RPMS/x86_64/volta-${release_version}-1.el7.x86_64.rpm\`"
echo ""
echo "Then run this to uninstall after verifying:"
echo "  \`sudo yum erase volta-${release_version}-1.el7.x86_64\`"
