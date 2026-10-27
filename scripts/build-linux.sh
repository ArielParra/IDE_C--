#!/bin/sh
set -e

echo "Building Linux binary..."
cargo test --locked --target-dir=/tmp/target
xvfb-run -a cargo test --locked --target-dir=/tmp/target gtk_semantic_views -- --ignored --test-threads=1
cargo build --locked --release --target-dir=/tmp/target
xvfb-run -a bash scripts/smoke-linux.sh /tmp/target/release/IDE_C--

mkdir -p /build/target/release
cp /tmp/target/release/IDE_C-- /build/target/release/IDE_C--
echo "Done: Binary placed in target/release/IDE_C--"
