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

    pub fn configure(
        &self,
        is_dark: bool,
    ) -> (
        Option<sourceview5::Language>,
        Option<sourceview5::StyleScheme>,
    ) {
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

        let scheme_name = if is_dark { "dark" } else { "light" };
        let scheme = self.style_manager.scheme(scheme_name);

        (language, scheme)
    }

    pub fn is_dark_mode() -> bool {
        #[cfg(windows)]
        {
            use winreg::enums::*;
            use winreg::RegKey;

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(personalize) = hkcu
                .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
            {
                if let Ok(apps_use_light_theme) =
                    personalize.get_value::<u32, _>("AppsUseLightTheme")
                {
                    return apps_use_light_theme == 0;
                }
            }
        }

        if let Some(display) = gtk::gdk::Display::default() {
            let settings = gtk::Settings::for_display(&display);
            return settings.is_gtk_application_prefer_dark_theme();
        }

        false
    }
}

impl Default for LanguageStyle {
    fn default() -> Self {
        Self::new()
    }
}
