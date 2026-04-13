use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Orientation};

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
        let (language, scheme) = language_style.configure();

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

        let menubar = build_menu(
            app,
            &window,
            &editor.buffer,
            editor.view,
            file_state,
            panels.lexic_view,
            panels.errors_view,
        );

        let windowbox = Box::new(Orientation::Vertical, 0);
        windowbox.append(&menubar);
        windowbox.append(&layout.container);

        window.set_titlebar(Some(&headerbar.widget));
        window.set_child(Some(&windowbox));

        Window { widget: window }
    }

    pub fn present(self) {
        self.widget.present();
    }
}
