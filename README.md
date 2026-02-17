# IDE_C--
IDE made in rust with gtk-rs for GUI

---

# Setup Rust and GTK-rs

## Windows (Diana)
```pwsh
winget install MSYS2.MSYS2
```

### Inside MSYS2 UCRT64
```sh
pacman -Syuu
pacman -S mingw-w64-ucrt-x86_64-rust mingw-w64-ucrt-x86_64-gtk4  mingw-w64-ucrt-x86_64-gtksourceview5 mingw-w64-ucrt-x86_64-pkgconf mingw-w64-ucrt-x86_64-gcc
```

## MacOs (Diana)

```sh
brew install rust gtk4 gtksourceview5 pkg-config
```


## alpine linux (Ariel)

```sh
doas apk add rust cargo build-base pkgconf gtk4-dev gtksourceview5-dev glib-dev gobject-introspection-dev cairo-dev pango-dev gdk-pixbuf-dev
```

## Arch (Miguel)
```sh
sudo pacman -S rust cargo base-devel pkgconf gtk4 gtksourceview5
```

# After Installation (Inside the project)
```sh
cargo add gtk4 --rename gtk
cargo run
```