#!/bin/bash

set -eufo pipefail

MYDIR="$(
    cd "$(dirname "$0")"
    pwd
)"
cd "${MYDIR}"

if uname -a | grep -v Darwin; then
    echo >&2 "ERROR: This script must be run on macOS"
    exit 1
fi

# Verify that we're on the right branch
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "${BRANCH}" != "main" ]]; then
    echo "ERROR: Releases can be done from the 'main' branch only"
    exit 1
fi

# Verify there are no outstanding changes
GIT_STATUS="$(git status --porcelain)"
if [[ -n "${GIT_STATUS}" ]]; then
    echo "ERROR: There are outstanding changes, make sure your working directory is clean before releasing"
    echo
    git status
    exit 1
fi

# Ensure we don't release broken things
./install.sh

echo "=="

# List changes since last release
git tag | cat

echo
echo "=="
echo "Enter new version number. The version number must be on macos-1.2.3 format."
read -r -p "New version number: " NEW_VERSION_NUMBER

echo Please enter "${NEW_VERSION_NUMBER}" again:
read -r -p "  Validate version: " VALIDATE_VERSION_NUMBER

if [[ "${NEW_VERSION_NUMBER}" != "${VALIDATE_VERSION_NUMBER}" ]]; then
    echo "Version numbers mismatch, never mind"
    exit 1
fi

if ! echo "${NEW_VERSION_NUMBER}" | grep -q -E '^macos-[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "ERROR: Version number must be on the form: macos-1.2.3: ${NEW_VERSION_NUMBER}"
    exit 1
fi

git tag "${NEW_VERSION_NUMBER}"

# Make the release build
./install.sh

# "--strip-components=2" removes the leading "/" and "Applications" from the
# archive paths
RELEASE_TAR="LoadViz-${NEW_VERSION_NUMBER}.tar.bz2"
tar cjf "${RELEASE_TAR}" --strip-components=2 /Applications/LoadViz.app

git push --tags

cat <<EOF
Release packaged and tagged.

Please create a new release on GitHub:
https://github.com/walles/loadviz/releases/new

And upload the build as well:
${RELEASE_TAR}

Then, please update the Homebrew formula as well:
https://github.com/walles/homebrew-johan/blob/main/Casks/loadviz.rb
EOF
