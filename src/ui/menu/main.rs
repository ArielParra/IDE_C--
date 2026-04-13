use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView};
use sourceview5::View as SourceView;
use std::cell::RefCell;
use std::rc::Rc;

use super::actions::ActionHandlers;
use super::builder::MenuBuilder;
use crate::models::FileState;

pub fn build_menu(
    app: &Application,
    window: &ApplicationWindow,
    buffer: &impl IsA<gtk::TextBuffer>,
    editor_view: SourceView,
    file_state: FileState,
    lex_view: Rc<RefCell<TextView>>,
    errors_view: Rc<RefCell<TextView>>,
) -> gtk::PopoverMenuBar {
    ActionHandlers::register_all(
        app,
        window,
        buffer,
        editor_view,
        file_state,
        lex_view,
        errors_view,
    );

    MenuBuilder::new()
        .add_file_menu()
        .add_edit_menu()
        .add_build_menu()
        .add_analysis_menu()
        .build()
}
