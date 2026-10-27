#!/bin/bash
set -e

echo "Starting macOS build inside Darling..."
darling shell -c "source ~/.bash_profile && cd /Volumes/SystemRoot/build && cargo build --release"
echo "Build finished."
