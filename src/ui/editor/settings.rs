use sourceview5::{Language, StyleScheme};

pub struct EditorSettings {
    pub language: Option<Language>,
    pub style_scheme: Option<StyleScheme>,
    pub show_line_numbers: bool,
    pub highlight_current_line: bool,
    pub monospace: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            language: None,
            style_scheme: None,
            show_line_numbers: true,
            highlight_current_line: true,
            monospace: true,
        }
    }
}

impl EditorSettings {
    pub fn with_language(mut self, language: Option<Language>) -> Self {
        self.language = language;
        self
    }

    pub fn with_style_scheme(mut self, scheme: Option<StyleScheme>) -> Self {
        self.style_scheme = scheme;
        self
    }
}
