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
    libgtk-4-dev libgtksourceview-5-dev xvfb xauth dbus-x11 xdotool fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:$PATH"
ENV PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig"

WORKDIR /build

COPY scripts/build-linux.sh /usr/local/bin/build-linux
RUN chmod +x /usr/local/bin/build-linux

CMD ["/usr/local/bin/build-linux"]
