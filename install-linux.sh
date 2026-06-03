#!/bin/sh
set -e

BIN_PATH="/usr/local/bin/IDE_C--"
DESKTOP_PATH="/usr/share/applications/com.ide_cmm.ide.desktop"
ICON_DIR="/usr/share/icons/hicolor"

echo "Installing IDE C--..."

cp target/release/IDE_C-- "$BIN_PATH"
cp src/resources/com.ide_cmm.ide.desktop "$DESKTOP_PATH"
cp -r src/resources/icons/hicolor/* "$ICON_DIR/"

gtk-update-icon-cache -f "$ICON_DIR" 2>/dev/null || true

echo "Done. You can now launch IDE C-- from your app launcher."
