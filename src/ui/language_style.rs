use sourceview5::{LanguageManager, StyleSchemeManager};

pub struct LanguageStyle {
    pub language_manager: LanguageManager,
    pub style_manager: StyleSchemeManager,
}

impl LanguageStyle {
    pub fn new() -> Self {
        Self {
            language_manager: LanguageManager::new(),
            style_manager: StyleSchemeManager::new(),
        }
    }

    pub fn configure(&self) -> (Option<sourceview5::Language>, Option<sourceview5::StyleScheme>) {
        let base = std::env::current_dir().unwrap();

        let lang_path = base.join("src/resources/language-specs");
        let style_path = base.join("src/resources/styles");

        let mut lang_paths = self.language_manager.search_path();
        lang_paths.push(lang_path.to_string_lossy().to_string().into());
        let lang_paths: Vec<&str> = lang_paths.iter().map(|s| s.as_str()).collect();
        self.language_manager.set_search_path(&lang_paths);

        let language = self.language_manager.language("cmm");

        let mut style_paths = self.style_manager.search_path();
        style_paths.push(style_path.to_string_lossy().to_string().into());
        let style_paths: Vec<&str> = style_paths.iter().map(|s| s.as_str()).collect();
        self.style_manager.set_search_path(&style_paths);

        let scheme = self.style_manager.scheme("dark");

        (language, scheme)
    }
}

impl Default for LanguageStyle {
    fn default() -> Self {
        Self::new()
    }
}
