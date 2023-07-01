#!/bin/bash

set -euf -o pipefail

# Update our icons
#
# This script is mostly from here:
# https://stackoverflow.com/a/20703594/473672

# Corner rounding from here:
# https://stackoverflow.com/a/1916256/473672
#
# "128" is a made up number.
CORNER_RADIUS=128

ICONSET="$(gmktemp -d -t 'loadviz-XXXXXXXX.iconset')"

MYDIR="$(
    cd "$(dirname "$0")"
    pwd
)"
cd "${MYDIR}"

RAWIMAGE=/tmp/loadviz-still.webp
(cd ../libloadviz && cargo run --bin=stillimage "${RAWIMAGE}")

MASK=/tmp/loadviz-mask.webp
convert -size 1024x1024 xc:none -draw "roundrectangle 0,0,1024,1024,${CORNER_RADIUS},${CORNER_RADIUS}" "${MASK}"

ICON1024=/tmp/loadviz-icon.webp
convert "${RAWIMAGE}" -matte "${MASK}" -compose DstIn -composite "${ICON1024}"

sips -s format png -z 16 16 "${ICON1024}" --out "${ICONSET}"/icon_16x16.png
sips -s format png -z 32 32 "${ICON1024}" --out "${ICONSET}"/icon_16x16@2x.png
sips -s format png -z 32 32 "${ICON1024}" --out "${ICONSET}"/icon_32x32.png
sips -s format png -z 64 64 "${ICON1024}" --out "${ICONSET}"/icon_32x32@2x.png
sips -s format png -z 128 128 "${ICON1024}" --out "${ICONSET}"/icon_128x128.png
sips -s format png -z 256 256 "${ICON1024}" --out "${ICONSET}"/icon_128x128@2x.png
sips -s format png -z 256 256 "${ICON1024}" --out "${ICONSET}"/icon_256x256.png
sips -s format png -z 512 512 "${ICON1024}" --out "${ICONSET}"/icon_256x256@2x.png
sips -s format png -z 512 512 "${ICON1024}" --out "${ICONSET}"/icon_512x512.png
cp "${ICON1024}" "${ICONSET}"/icon_512x512@2x.png

iconutil -c icns -o loadviz.icns "${ICONSET}"

rm -R "${ICONSET}"
