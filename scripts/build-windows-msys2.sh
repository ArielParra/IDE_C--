#!/bin/bash
set -e

cd /opt/quasi-msys2
source env/vars.src
cd /build

# Force cargo to use the native pkg-config for the host build script (build.rs)
export PKG_CONFIG_x86_64_unknown_linux_gnu="pkg-config"
export PKG_CONFIG_PATH_x86_64_unknown_linux_gnu="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig"
export PKG_CONFIG_SYSROOT_DIR_x86_64_unknown_linux_gnu="/"

cargo build --locked --release
mkdir -p /tmp/output/src
cp target/${CARGO_BUILD_TARGET}/release/IDE_C--.exe /tmp/output/
cp -r src/resources /tmp/output/src/
cp /ucrt64/bin/*.dll /tmp/output/ 2>/dev/null || cp /opt/quasi-msys2/root/ucrt64/bin/*.dll /tmp/output/
cd /tmp/output
mkdir -p /build/releases
zip -r /build/releases/IDE_C--_windows_msys2.zip .
echo "Done: releases/IDE_C--_windows_msys2.zip"
