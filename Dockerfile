# =============================================
# Stage 1: Build on Alpine Linux
# =============================================
FROM alpine:edge AS builder

RUN apk add --no-cache \
    rust cargo \
    build-base pkgconf \
    gtk4.0-dev gtksourceview5-dev glib-dev \
    gobject-introspection-dev cairo-dev pango-dev gdk-pixbuf-dev \
    gcompat libc6-compat

WORKDIR /build
COPY . .

RUN cargo build --release

# =============================================
# Stage 2: Minimal runtime image
# =============================================
FROM alpine:edge

RUN apk add --no-cache \
    rust cargo build-base pkgconf \
    gtk4.0-dev gtksourceview5-dev glib-dev \
    gobject-introspection-dev cairo-dev pango-dev gdk-pixbuf-dev \
    gcompat libc6-compat \
    adwaita-icon-theme font-noto

COPY --from=builder /build/target/release/IDE_C-- /usr/local/bin/IDE_C--
COPY --from=builder /build/src/styles.css /usr/local/share/IDE_C--/src/styles.css
COPY --from=builder /build/src/resources /usr/local/share/IDE_C--/src/resources

WORKDIR /usr/local/share/IDE_C--

CMD ["IDE_C--"]
