use gtk::prelude::*;
use gtk::ScrolledWindow;
use sourceview5::traits::{BufferExt, ViewExt};
use sourceview5::{Buffer, LanguageManager, StyleSchemeManager, View};

use super::settings::EditorSettings;

pub struct EditorComponents {
    pub buffer: Buffer,
    pub view: View,
    pub container: ScrolledWindow,
}

pub fn create_editor(
    _lm: LanguageManager,
    _sm: StyleSchemeManager,
    settings: &EditorSettings,
) -> EditorComponents {
    let buffer = Buffer::new(None);

    if let Some(language) = &settings.language {
        buffer.set_language(Some(language));
    }

    if let Some(scheme) = &settings.style_scheme {
        buffer.set_style_scheme(Some(scheme));
    }

    buffer.set_highlight_syntax(true);

    let view = View::with_buffer(&buffer);
    view.set_show_line_numbers(settings.show_line_numbers);
    view.set_highlight_current_line(settings.highlight_current_line);
    view.set_monospace(settings.monospace);

    let scrolled = ScrolledWindow::builder()
        .child(&view)
        .vexpand(true)
        .hexpand(true)
        .build();

    EditorComponents {
        buffer,
        view,
        container: scrolled,
    }
}
