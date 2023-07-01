#!/bin/bash

set -euf -o pipefail

# Update our icons
#
# This script is mostly from here:
# https://stackoverflow.com/a/20703594/473672

ICONSET="$(gmktemp -d -t 'loadviz-XXXXXXXX.iconset')"

MYDIR="$(
    cd "$(dirname "$0")"
    pwd
)"
cd "${MYDIR}"

BASEIMAGE=/tmp/loadviz-still.webp
(cd ../libloadviz && cargo run --bin=stillimage "${BASEIMAGE}")

sips -s format png -z 16 16 "${BASEIMAGE}" --out "${ICONSET}"/icon_16x16.png
sips -s format png -z 32 32 "${BASEIMAGE}" --out "${ICONSET}"/icon_16x16@2x.png
sips -s format png -z 32 32 "${BASEIMAGE}" --out "${ICONSET}"/icon_32x32.png
sips -s format png -z 64 64 "${BASEIMAGE}" --out "${ICONSET}"/icon_32x32@2x.png
sips -s format png -z 128 128 "${BASEIMAGE}" --out "${ICONSET}"/icon_128x128.png
sips -s format png -z 256 256 "${BASEIMAGE}" --out "${ICONSET}"/icon_128x128@2x.png
sips -s format png -z 256 256 "${BASEIMAGE}" --out "${ICONSET}"/icon_256x256.png
sips -s format png -z 512 512 "${BASEIMAGE}" --out "${ICONSET}"/icon_256x256@2x.png
sips -s format png -z 512 512 "${BASEIMAGE}" --out "${ICONSET}"/icon_512x512.png
cp "${BASEIMAGE}" "${ICONSET}"/icon_512x512@2x.png

iconutil -c icns -o loadviz.icns "${ICONSET}"

rm -R "${ICONSET}"
