#!/bin/bash

# Set version string and copyright from git describe
#
# See: https://developer.apple.com/library/content/documentation/General/Reference/InfoPlistKeyReference/Articles/CoreFoundationKeys.html

set -e
set -x

if [ ! "${INFO_PLIST}" ] ; then
  INFO_PLIST="${TARGET_BUILD_DIR}/${INFOPLIST_PATH}"
fi

YEAR="$(git log -1 --format='%cd' --date=format:'%Y')"
COPYRIGHT="Copyright 2023-${YEAR} johan.walles@gmail.com"
/usr/libexec/PlistBuddy -c "Add :NSHumanReadableCopyright string \"$COPYRIGHT\"" "${INFO_PLIST}" ||
/usr/libexec/PlistBuddy -c "Set :NSHumanReadableCopyright \"$COPYRIGHT\"" "${INFO_PLIST}"

# Example: 0.3.0
VERSION=$(git describe --tags --match='macos-*' --dirty | cut -d- -f2)

# I don't know the difference between CFBundleShortVersionString and CFBundleVersion.
# We do both here to be on the safe side.
/usr/libexec/PlistBuddy -c "Add :CFBundleShortVersionString string \"$VERSION\"" "${INFO_PLIST}" ||
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString \"$VERSION\"" "${INFO_PLIST}"
/usr/libexec/PlistBuddy -c "Add :CFBundleVersion string \"$VERSION\"" "${INFO_PLIST}" ||
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion \"$VERSION\"" "${INFO_PLIST}"

# Example: aaf83526
GITHASH=$(git rev-parse --verify --short=8 HEAD)
if [[ $(git diff HEAD --stat) != '' ]]; then
  GITHASH="$GITHASH-dirty"
fi
/usr/libexec/PlistBuddy -c "Add :GitHash string $GITHASH" "$INFO_PLIST" ||
  /usr/libexec/PlistBuddy -c "Set :GitHash $GITHASH" "$INFO_PLIST"

# Example: 0.10.2-8-gaaf8352-dirty
GITDESCRIBE=$(git describe --tags --match='macos-*' --dirty| sed 's/^macos-//')
/usr/libexec/PlistBuddy -c "Add :GitDescribe string $GITDESCRIBE" "$INFO_PLIST" ||
  /usr/libexec/PlistBuddy -c "Set :GitDescribe $GITDESCRIBE" "$INFO_PLIST"
