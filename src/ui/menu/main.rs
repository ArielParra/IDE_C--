use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, TextView, SearchBar, SearchEntry};
use sourceview5::View as SourceView;
use std::cell::RefCell;
use std::rc::Rc;

use super::actions::ActionHandlers;
use super::builder::MenuBuilder;
use super::navigator::{ErrorNavigator, LexicNavigator, AstNavigator};
use crate::models::FileState;

pub fn build_menu(
    app: &Application,
    window: &ApplicationWindow,
    buffer: &impl IsA<gtk::TextBuffer>,
    editor_view: SourceView,
    file_state: FileState,
    lex_view: Rc<RefCell<TextView>>,
    errors_view: Rc<RefCell<TextView>>,
    syntax_errors_view: Rc<RefCell<TextView>>,
    ast_view: Rc<RefCell<crate::ui::panels::AstView>>,
    debug_notebook: gtk::Notebook,
    errors_notebook: gtk::Notebook,
    search_bar: SearchBar,
    search_entry: SearchEntry,
) -> gtk::gio::Menu {
    let editor_buffer: gtk::TextBuffer = buffer.as_ref().clone();

    ErrorNavigator::connect_error_click(&errors_view, &editor_buffer, &editor_view);
    ErrorNavigator::connect_error_click(&syntax_errors_view, &editor_buffer, &editor_view);
    LexicNavigator::connect_position_click(&lex_view, &editor_buffer, &editor_view);
    AstNavigator::connect_ast_click(&ast_view, &editor_buffer, &editor_view);

    ActionHandlers::register_all(
        app,
        window,
        buffer,
        editor_view.clone(),
        file_state,
        lex_view,
        errors_view,
        syntax_errors_view,
        ast_view,
        debug_notebook,
        errors_notebook,
        search_bar,
        search_entry,
    );

    MenuBuilder::new()
        .add_file_menu()
        .add_edit_menu()
        .add_build_menu()
        .add_analysis_menu()
        .build()
}
