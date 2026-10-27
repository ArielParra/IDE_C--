#!/bin/bash
set -e

echo "Building Linux binary for AppImage..."
cargo build --locked --release --target-dir=/tmp/target

# Prepare AppDir
export APPDIR=/tmp/AppDir
mkdir -p $APPDIR/usr/bin
mkdir -p $APPDIR/usr/share/applications
mkdir -p $APPDIR/usr/share/icons/hicolor/256x256/apps

cp /tmp/target/release/IDE_C-- $APPDIR/usr/bin/
cp src/resources/com.ide_cmm.ide.desktop $APPDIR/usr/share/applications/
cp -r src/resources/icons/hicolor/* $APPDIR/usr/share/icons/hicolor/

echo "Running linuxdeploy..."
export LDAI_OUTPUT=/build/target/release/IDE_C--.AppImage
mkdir -p /build/target/release

# The plugin requires these to locate the AppDir correctly
export VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n 1)"
export LINUXDEPLOY_OUTPUT_VERSION="$VERSION"
export DEPLOY_GTK_VERSION=4

# Using the extracted AppRun as the linuxdeploy executable
/opt/linuxdeploy/AppRun \
    --appdir $APPDIR \
    --desktop-file src/resources/com.ide_cmm.ide.desktop \
    --icon-file src/resources/icons/hicolor/256x256/apps/com.ide_cmm.ide.png \
    --plugin gtk \
    --output appimage

test -s "$LDAI_OUTPUT"
APPIMAGE_EXTRACT_AND_RUN=1 xvfb-run -a bash scripts/smoke-linux.sh "$LDAI_OUTPUT"
chown "$(stat -c "%u:%g" /build)" "$LDAI_OUTPUT"

echo "Done: target/release/IDE_C--.AppImage"
