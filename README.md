# IDE C--

**Version 1.3.0 — Semantic Analyzer** · [Release notes](docs/releases/1.3.md)

A modern, cross-platform **Integrated Development Environment** for the **C--** programming language, built with [Rust](https://www.rust-lang.org/) and [GTK4](https://gtk.org/) via [gtk-rs](https://gtk-rs.org/).

---

## Features

- **Source Code Editor** — Powered by [GtkSourceView5](https://wiki.gnome.org/Projects/GtkSourceView) with syntax highlighting, line numbers, and auto-indentation.
- **Lexical Analysis** — Tokenizes C-- source code, displays colored token output, and reports lexical errors with clickable line/column navigation.
- **Syntax Analysis** — Builds an Abstract Syntax Tree (AST) from tokens and renders it as an interactive, expandable tree with Unicode box-drawing connectors (`├──`, `└──`, `│`).
- **Semantic Analysis** — Analyze the current editor text, including unsaved changes, without a saved token file. Inspect a separately annotated AST with types, constant values and numeric conversions.
- **Symbol Table** — Display declarations, scopes, reference positions and logical storage offsets from the actual analyzer, with navigation and copying.
- **Error Navigation** — Click on any error or AST node to jump directly to the corresponding line and column in the editor.
- **Token Persistence** — The lexical action saves `*.c--.tokens` as an export. Syntax and Semantic Analysis refresh the analysis directly from the current buffer; editing clears prior results.
- **Cross-Platform** — Release binaries for x86-64 Linux and Windows (MSYS2 and MinGW); native macOS source builds remain available.

## Architecture

```
src/
├── main.rs                  # Application entry point, CSS & theme loading
├── styles.css               # GTK4 custom theme overrides
├── compiler/
│   ├── lexer/               # Lexical analyzer (tokenizer, handlers, errors)
│   ├── parser/              # Syntax analyzer (core, expressions, statements)
│   ├── semantic/            # Annotated AST, symbol registry, types and constants
│   └── analysis.rs          # Current-buffer pipeline for all three phases
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
    │   ├── semantic_action.rs # Current-buffer analysis and diagnostics
    │   └── navigator.rs      # Click-to-code navigation
    └── panels/
        ├── ast_view.rs       # AST tree (GObject + ListView + TreeListModel)
        ├── symbols_view.rs   # Symbol table with navigation and copying
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
cargo build --locked --release
```

The binary will be at `target/release/IDE_C--`.

### Installation

On Linux, you can install the application locally without `sudo` by running:

```sh
make install
```

This will place the executable in `~/.local/bin`, the desktop entry in `~/.local/share/applications`, and the icons in `~/.local/share/icons/hicolor`. To uninstall, run `make uninstall`.

> **Pre-compiled Binaries:** You can also download the pre-compiled binary files directly from the [Releases section of this GitHub repository](https://github.com/ArielParra/IDE_C--/releases).

---

## Docker - Cross Compilations

### Linux Build (Native)

Build the project inside a Debian Trixie slim container. This keeps the exported binary on glibc and provides GTK 4.10+ for the `gtk4` Rust crate:

```sh
docker build -t ide-cmm-linux .
```

To compile and output the Linux binary to your host machine:

```sh
docker run --rm -v "$(pwd)":/build ide-cmm-linux
```

This will compile the project in the container and automatically place the executable at `target/release/IDE_C--`. Run it on the host with:

```sh
./target/release/IDE_C--
```

The host system still needs GTK4 4.10+, GtkSourceView5 and the native runtime dependencies. Release Linux builds use the Debian 13 (Trixie) glibc baseline; older distributions may need a source build.

### Linux AppImage (Portable)

If you want to create a standalone, portable Linux executable that bundles GTK4 and its dependencies (an `.AppImage`), you can use the AppImage builder:

```sh
docker build -f Dockerfile.appimage -t ide-cmm-appimage .
docker run --rm -v "$(pwd)":/build ide-cmm-appimage
```

This will compile the project and use `linuxdeploy` to generate an AppImage file. The final `IDE_C--.AppImage` is written to `target/release/`. It bundles GTK4 and application libraries but relies on the host glibc. If FUSE is unavailable, run it with `APPIMAGE_EXTRACT_AND_RUN=1 ./target/release/IDE_C--.AppImage`.

### Windows Cross-Compilation

This project includes two Docker-based Windows cross-compilation options:

```sh
docker build -f Dockerfile.msys2 -t ide-cmm-msys2 .
docker run --rm -v "$(pwd)":/build ide-cmm-msys2
```

```sh
docker build -f Dockerfile.mingw -t ide-cmm-mingw .
docker run --rm -v "$(pwd)":/build ide-cmm-mingw
```

**Comparison of Windows builds:**
- **MSYS2:** Recommended UCRT64 build using [quasi-msys2](https://github.com/HolyBlackCat/quasi-msys2) packages.
- **MinGW:** Alternative build using Fedora's MinGW toolchain.

Package sizes and DLL counts depend on the toolchain versions used for each release.

Both generate a portable `.zip` archive (`IDE_C--_windows_msys2.zip` and `IDE_C--_windows_mingw.zip` respectively) in the `releases/` directory.

> **Note on Portability:** These Windows builds are completely standalone and portable. All required dynamic libraries (`.dll` files for GTK4, GLib, Pango, etc.) are bundled directly alongside the `.exe` inside the archive. You can extract the folder anywhere on a Windows machine and run it immediately without needing to install MSYS2 or any global dependencies.

### macOS Cross-Compilation (Experimental)

> **WARNING:** This macOS cross-compilation image relies on [Darling](https://www.darlinghq.org/), which requires the `darling-mach` kernel module to be compiled and loaded on your physical Linux host. This Docker container **will fail** if you run it on native Windows, macOS, or a Linux host without the kernel module. Additionally, Homebrew support inside Darling is highly experimental and compiling heavy GUI frameworks like GTK4 may fail.

```sh
# On your Linux host, ensure the module is loaded:
sudo modprobe darling-mach

# Build and run the Darling image (requires --privileged):
docker build -f Dockerfile.macos -t ide-cmm-macos .
docker run --rm --privileged -v "$(pwd)":/build ide-cmm-macos
```

The macOS binary will be output to `target/release/IDE_C--`. For a truly production-ready macOS build, it is highly recommended to use a native macOS machine, GitHub Actions or a Virtual Machine instead of Docker.

---

## Semantic Analysis

Choose **Semantic Analysis → Run Semantic Analysis**. The current editor buffer feeds the lexical, syntax and semantic phases together. The Syntax Analysis action refreshes the same results and selects the original tree. Compare **Syntax** with **Semantic**, inspect declarations in **Hash Table**, and click diagnostics to navigate to source. Editing invalidates previous results. Lexical errors stop later phases; syntax recovery produces explicitly partial results.

See [Phase 3 semantic rules](docs/phase3_semantic_rules.md) for attributes, symbol storage, conversion rules and examples. The existing `semantic.c--` demonstration intentionally contains errors and remains unchanged.

## Tests

```sh
cargo test --locked
# GTK integration: requires a real display or Xvfb.
xvfb-run -a cargo test --locked gtk_semantic_views -- --ignored --test-threads=1
python3 -m unittest discover -s scripts -p 'test_*.py'
```

The native Docker builder runs the Rust suite, GTK integration and a visible-window startup check before exporting its release binary.

## Release Workflow

The **Release 1.3 - Semantic Analyzer** workflow builds all four packages on separate GitHub runners from the exact workflow commit. Its Windows jobs launch each extracted ZIP with a system-only PATH, and Linux builders perform visible-window checks under Xvfb.

Pushing the `release/1.3` branch runs build and startup checks without creating a tag or release. A manual run prepares a draft by default. Select **publish_release**, or push the explicit `1.3` tag, to publish it as stable and Latest after all checks pass. The tag is pushed only after the branch validation succeeds. Published releases and tags pointing to another commit are never overwritten. `1.2` remains unchanged. Creating or dispatching a workflow requires repository access; the workflow uses its own restricted GitHub token for the draft and publication.

All assets must pass ZIP/ELF/PE validation, runtime DLL closure checks and source-provenance checks. The workflow generates `SHA256SUMS`, downloads the draft assets and compares their checksums before publication, then downloads and verifies the public assets again. Build manifests are retained as workflow artifacts.

Local staging/verification helpers are available through `python3 scripts/release.py --help`. These scripts require Python 3.11 or newer. The four release assets are `IDE_C--`, `IDE_C--.AppImage`, `IDE_C--_windows_msys2.zip` and `IDE_C--_windows_mingw.zip`.

---

## License

This project is for academic purposes.
