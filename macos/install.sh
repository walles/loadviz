#!/bin/bash

# Install LoadViz from source into /Applications

set -eufo pipefail

echo "INFO: Checking build environment..."

# Verify we're on Darwin
if [ "$(uname)" != "Darwin" ]; then
    echo >&2 "ERROR: LoadViz only works on macOS"
    exit 1
fi

# Verify XCode is installed
XCODEDIR="/Applications/Xcode.app/Contents/Developer"
if [ ! -d "${XCODEDIR}" ]; then
    echo >&2 "ERROR: Need XCode to build LoadViz. Install XCode from here then try again: "
    echo >&2 "  https://apps.apple.com/se/app/xcode/id497799835?mt=12"

    exit
fi

if [ "$(xcode-select --print-path)" != "${XCODEDIR}" ]; then
    echo
    echo "Picking a complete XCode installation to build from, please enter your password when prompted:"
    echo
    echo "\$ sudo xcode-select -s ${XCODEDIR}"
    sudo xcode-select -s "${XCODEDIR}"
fi

# Verify we can xcrun git and xcodebuild
if xcrun git --version >/dev/null && xcrun xcodebuild -version >/dev/null; then
    echo "INFO: Command line dev tools found, proceeding..."
elif xcode-select --install; then
    echo >&2 "WARNING: Developer tools not found, trying to get them installed."
    echo >&2
    echo >&2 "After they are in place, run this script again!"
    echo >&2
    echo >&2 "If the automatic install doesn't work, check here:"
    echo >&2 "https://osxdaily.com/2014/02/12/install-command-line-tools-mac-os-x/"

    exit
else
    # xcode-select failed: https://stackoverflow.com/a/47804075/473672
    echo >&2
    echo >&2 "ERROR: Installing development tools failed. Try this manually:"
    echo >&2
    echo >&2 "  sudo rm -rf /Library/Developer/CommandLineTools && xcode-select --install"
    echo >&2
    echo >&2 "This will remove your current development tools and reinstall them."
    echo >&2 "Source: <https://stackoverflow.com/a/47804075/473672>"
    echo >&2
    echo >&2 "Then run this script again."

    exit 1
fi

MYDIR="$(
    cd "$(dirname "$0")"
    pwd
)"
echo "INFO: Installing from: ${MYDIR}"
cd "${MYDIR}"

echo
echo "INFO: Now building, this can take 30+ seconds and a lot of text can scroll by..."
date
TARGET_BUILD_DIR=$(mktemp -d -t loadviz-build)
time xcrun xcodebuild \
    build \
    -quiet \
    -project loadviz.xcodeproj \
    -configuration Release \
    -target "LoadViz" \
    CONFIGURATION_BUILD_DIR="${TARGET_BUILD_DIR}"

echo "INFO: Build done, now installing..."

LOADVIZ_APP="${TARGET_BUILD_DIR}/LoadViz.app"

# Back up any existing installation
if [ -e "/Applications/LoadViz.app" ]; then
    rm -rf "/Applications/LoadViz.app.old"
    mv "/Applications/LoadViz.app" "/Applications/LoadViz.app.old"
fi

# Install!
mv "${LOADVIZ_APP}" "/Applications/"

# Start the menu bar app
echo "INFO: Now launching the Menu Bar app..."
sleep 3 # Without this the open command sometimes doesn't find our new app
open "/Applications/LoadViz.app"
