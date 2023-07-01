#!/bin/sh

set -euf -o pipefail

# Turn our screenshot into an icon
#
# This script is mostly from here:
# https://stackoverflow.com/a/20703594/473672

ICONSET="$(gmktemp -d -t 'loadviz-XXXXXXXX.iconset')"

# FIXME: Update / create this before using it? With a higher resolution?
SCREENSHOT=../libloadviz/screenshot.webp

MYDIR="$(cd "$(dirname "$0")"; pwd)"
cd "${MYDIR}"

sips -s format png -z 16 16     "${SCREENSHOT}" --out "${ICONSET}"/icon_16x16.png
sips -s format png -z 32 32     "${SCREENSHOT}" --out "${ICONSET}"/icon_16x16@2x.png
sips -s format png -z 32 32     "${SCREENSHOT}" --out "${ICONSET}"/icon_32x32.png
sips -s format png -z 64 64     "${SCREENSHOT}" --out "${ICONSET}"/icon_32x32@2x.png
sips -s format png -z 128 128   "${SCREENSHOT}" --out "${ICONSET}"/icon_128x128.png
sips -s format png -z 256 256   "${SCREENSHOT}" --out "${ICONSET}"/icon_128x128@2x.png
sips -s format png -z 256 256   "${SCREENSHOT}" --out "${ICONSET}"/icon_256x256.png
sips -s format png -z 512 512   "${SCREENSHOT}" --out "${ICONSET}"/icon_256x256@2x.png
sips -s format png -z 512 512   "${SCREENSHOT}" --out "${ICONSET}"/icon_512x512.png
cp "${SCREENSHOT}" "${ICONSET}"/icon_512x512@2x.png

iconutil -c icns -o loadviz.icns "${ICONSET}"

rm -R "${ICONSET}"
