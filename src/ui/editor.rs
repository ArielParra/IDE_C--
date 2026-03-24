// ================= EDITOR =================
use gtk::prelude::*;
use gtk::{ScrolledWindow};
use sourceview5::traits::{BufferExt, ViewExt};
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};

// ================= EDITOR =================

// Crea el editor de código: buffer, vista y contenedor con scroll.
// Aplica lenguaje, esquema de colores, números de línea y resaltado de línea actual.

pub fn create_editor(
    _lm: LanguageManager,
    _sm: StyleSchemeManager,
    lang: Option<sourceview5::Language>,
    scheme: Option<sourceview5::StyleScheme>,
) -> (Buffer, View, ScrolledWindow) {

    let buffer = Buffer::new(None);

    if let Some(language) = lang {
        buffer.set_language(Some(&language));
    }

    if let Some(scheme) = scheme {
        buffer.set_style_scheme(Some(&scheme));
    }

    buffer.set_highlight_syntax(true);

    let view = View::with_buffer(&buffer);

    view.set_show_line_numbers(true);
    view.set_highlight_current_line(true);
    view.set_monospace(true);

    let scrolled = ScrolledWindow::builder()
        .child(&view)
        .vexpand(true)
        .hexpand(true)
        .build();

    (buffer, view, scrolled)
}
