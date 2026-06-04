# =============================================
# Linux native compilation
# Usage:
#   docker build -t ide-cmm-linux .
#   docker run --rm -v "$(pwd)":/build ide-cmm-linux
#
# Output: IDE_C-- binary in target/release/
# =============================================
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config curl ca-certificates \
    libgtk-4-dev libgtksourceview-5-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:$PATH"
ENV PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig"

WORKDIR /build

RUN cat > /usr/local/bin/build-linux << 'SH'
#!/bin/sh
set -e

echo "Building Linux binary..."
cargo build --release --target-dir=/tmp/target

mkdir -p /build/target/release
cp /tmp/target/release/IDE_C-- /build/target/release/IDE_C--
echo "Done: Binary placed in target/release/IDE_C--"
SH

RUN chmod +x /usr/local/bin/build-linux

CMD ["/usr/local/bin/build-linux"]
