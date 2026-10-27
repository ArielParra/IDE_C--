#!/bin/sh
set -e

cargo build --locked --release --target x86_64-pc-windows-gnu
mkdir -p /tmp/output/src
cp target/x86_64-pc-windows-gnu/release/IDE_C--.exe /tmp/output/
cp -r src/resources /tmp/output/src/
cp /usr/x86_64-w64-mingw32/sys-root/mingw/bin/*.dll /tmp/output/
cd /tmp/output
mkdir -p /build/releases
zip -r /build/releases/IDE_C--_windows_mingw.zip .
echo "Done: releases/IDE_C--_windows_mingw.zip"
