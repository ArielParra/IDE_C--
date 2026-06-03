use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation, SearchBar, SearchEntry};

use super::editor::{create_editor, EditorSettings};
use super::headerbar::IDEHeaderBar;
use super::menu::build_menu;
use super::panels::{create_panels, Layout};
use crate::models::new_file_state;
use crate::ui::language_style::LanguageStyle;

pub struct Window {
    pub widget: ApplicationWindow,
}

impl Window {
    pub fn build(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("IDE_C--")
            .default_width(900)
            .default_height(600)
            .build();

        let language_style = LanguageStyle::new();
        let is_dark = LanguageStyle::is_dark_mode();
        let (language, scheme) = language_style.configure(is_dark);

        let settings = EditorSettings::default()
            .with_language(language)
            .with_style_scheme(scheme);

        let editor = create_editor(
            language_style.language_manager,
            language_style.style_manager,
            &settings,
        );

        let panels = create_panels();
        let layout = Layout::new(&editor.container, &panels.debug, &panels.errors);
        let headerbar = IDEHeaderBar::new(app);
        let file_state = new_file_state();

        let search_bar = SearchBar::builder()
            .key_capture_widget(&window)
            .build();
        let search_entry = SearchEntry::builder()
            .placeholder_text("Search text...")
            .hexpand(true)
            .build();
        search_bar.set_child(Some(&search_entry));

        let menubar_model = build_menu(
            app,
            &window,
            &editor.buffer,
            editor.view,
            file_state,
            panels.lexic_view,
            panels.errors_view,
            panels.syntax_errors_view,
            panels.ast_view.clone(),
            panels.debug.clone(),
            panels.errors.clone(),
            search_bar.clone(),
            search_entry.clone(),
        );

        app.set_menubar(Some(&menubar_model));
        window.set_show_menubar(true);

        let windowbox = Box::new(Orientation::Vertical, 0);
        windowbox.append(&search_bar);
        windowbox.append(&layout.container);

        window.set_titlebar(Some(&headerbar.widget));
        window.set_child(Some(&windowbox));

        Window { widget: window }
    }

    pub fn present(self) {
        self.widget.present();
    }
}
