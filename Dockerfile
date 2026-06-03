# =============================================
# Stage 1: Build a glibc Linux binary
# =============================================
FROM debian:trixie-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config curl ca-certificates \
    libgtk-4-dev libgtksourceview-5-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:$PATH"
ENV PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig"

WORKDIR /build
COPY . .

RUN cargo build --release

# =============================================
# Stage 2: Small glibc runtime/export image
# =============================================
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libgtk-4-1 libgtksourceview-5-0 \
    adwaita-icon-theme fonts-noto-core \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/IDE_C-- /usr/local/bin/IDE_C--
COPY --from=builder /build/src/resources /usr/local/share/IDE_C--/src/resources
COPY --from=builder /build/src/resources/icons/hicolor /usr/share/icons/hicolor
COPY --from=builder /build/src/resources/com.ide_cmm.ide.desktop /usr/share/applications/com.ide_cmm.ide.desktop

RUN gtk-update-icon-cache -f /usr/share/icons/hicolor || true

WORKDIR /usr/local/share/IDE_C--

CMD ["IDE_C--"]
