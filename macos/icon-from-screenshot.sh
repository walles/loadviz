#!/bin/bash

set -euf -o pipefail

# Update our icons
#
# This script is mostly from here:
# https://stackoverflow.com/a/20703594/473672

# Corner rounding from here: https://stackoverflow.com/a/1916256/473672
#
# Experiments show that "256" seems to be what the Safari icon is using, so that
# should be good for us as well.
CORNER_RADIUS=256

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

# This makes us match the Safari icon, which seems to be 832x832 centered on a
# 1024x1024 canvas.
ICON832=/tmp/loadviz-icon-832.webp
convert "${ICON1024}" -resize 832x832 -gravity center -background transparent -extent 1024x1024 "${ICON832}"

sips -s format png -z 16 16 "${ICON832}" --out "${ICONSET}"/icon_16x16.png
sips -s format png -z 32 32 "${ICON832}" --out "${ICONSET}"/icon_16x16@2x.png
sips -s format png -z 32 32 "${ICON832}" --out "${ICONSET}"/icon_32x32.png
sips -s format png -z 64 64 "${ICON832}" --out "${ICONSET}"/icon_32x32@2x.png
sips -s format png -z 128 128 "${ICON832}" --out "${ICONSET}"/icon_128x128.png
sips -s format png -z 256 256 "${ICON832}" --out "${ICONSET}"/icon_128x128@2x.png
sips -s format png -z 256 256 "${ICON832}" --out "${ICONSET}"/icon_256x256.png
sips -s format png -z 512 512 "${ICON832}" --out "${ICONSET}"/icon_256x256@2x.png
sips -s format png -z 512 512 "${ICON832}" --out "${ICONSET}"/icon_512x512.png
cp "${ICON832}" "${ICONSET}"/icon_512x512@2x.png

iconutil -c icns -o loadviz.icns "${ICONSET}"

rm -R "${ICONSET}"
