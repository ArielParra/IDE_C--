# IDE C--

A modern, cross-platform **Integrated Development Environment** for the **C--** programming language, built with [Rust](https://www.rust-lang.org/) and [GTK4](https://gtk.org/) via [gtk-rs](https://gtk-rs.org/).

---

## Features

- **Source Code Editor** — Powered by [GtkSourceView5](https://wiki.gnome.org/Projects/GtkSourceView) with syntax highlighting, line numbers, and auto-indentation.
- **Lexical Analysis** — Tokenizes C-- source code, displays colored token output, and reports lexical errors with clickable line/column navigation.
- **Syntax Analysis** — Builds an Abstract Syntax Tree (AST) from tokens and renders it as an interactive, expandable tree with Unicode box-drawing connectors (`├──`, `└──`, `│`).
- **Error Navigation** — Click on any error or AST node to jump directly to the corresponding line and column in the editor.
- **Token Persistence** — Tokens are saved per-file (`*.c--.tokens`), allowing syntax analysis to be run independently.
- **Cross-Platform** — Runs on Linux, Windows (via MSYS2), and macOS.

## Architecture

```
src/
├── main.rs                  # Application entry point, CSS & theme loading
├── styles.css               # GTK4 custom theme overrides
├── compiler/
│   ├── lexer/               # Lexical analyzer (tokenizer, handlers, errors)
│   └── parser/              # Syntax analyzer (core, expressions, statements)
├── models/
│   └── ast.rs               # AST data structure
├── file_manager/
│   └── file_ops.rs          # File I/O (new, open, save, save-as)
└── ui/
    ├── window.rs             # Main window builder
    ├── editor/               # Source editor setup
    ├── headerbar/            # Menu bar
    ├── menu/
    │   ├── actions.rs        # GIO action handlers (lexical, syntax, compile)
    │   └── navigator.rs      # Click-to-code navigation
    └── panels/
        ├── ast_view.rs       # AST tree (GObject + ListView + TreeListModel)
        ├── notebook.rs       # Tab panels
        └── layout.rs         # Panel layout
```

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (2024 edition) |
| GUI Framework | GTK4 (gtk-rs v0.7) |
| Code Editor | GtkSourceView5 |
| AST Rendering | GObject subclassing + `ListView` + `TreeListModel` |

---

## Setup

### Windows

```pwsh
winget install MSYS2.MSYS2
```

Inside **MSYS2 UCRT64**:

```sh
pacman -Syuu
pacman -S mingw-w64-ucrt-x86_64-rust mingw-w64-ucrt-x86_64-gtk4 mingw-w64-ucrt-x86_64-gtksourceview5 mingw-w64-ucrt-x86_64-pkgconf mingw-w64-ucrt-x86_64-gcc
```

### macOS

```sh
brew install rust gtk4 gtksourceview5 pkg-config
```

### Alpine Linux

```sh
doas apk add rust cargo build-base pkgconf gtk4.0-dev gtksourceview5-dev glib-dev gobject-introspection-dev cairo-dev pango-dev gdk-pixbuf-dev gcompat libc6-compat
```

### Fedora Linux

```sh
sudo dnf group install "Development Tools"
sudo dnf install rust cargo gtk4-devel gtksourceview5-devel pkg-config
```

### Arch Linux

```sh
sudo pacman -S rust cargo base-devel pkgconf gtk4 gtksourceview5
```

---

## Build & Run

```sh
cargo run
```

To create an optimized release build:

```sh
cargo build --release
```

The binary will be at `target/release/IDE_C--`.

---

## Docker

### Linux Build (Alpine)

Build the project inside a lightweight Alpine container:

```sh
docker build -t ide-cmm-linux .
```

### Windows Cross-Compile

Cross-compile for Windows using the [gtk4-cross](https://github.com/MGlolenstine/gtk4-cross) image:

```sh
docker build -f Dockerfile.cross -t ide-cmm-cross .
docker run -v $(pwd):/mnt ide-cmm-cross
```

This generates `IDE_C--_windows.zip` in your project directory with the `.exe` and all required DLLs.

---

## License

This project is for academic purposes.