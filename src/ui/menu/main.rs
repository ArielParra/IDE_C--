use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, SearchBar, SearchEntry};
use sourceview5::View as SourceView;

use super::actions::ActionHandlers;
use super::builder::MenuBuilder;
use super::navigator::{AstNavigator, ErrorNavigator, LexicNavigator};
use crate::models::FileState;

pub fn build_menu(
    app: &Application,
    window: &ApplicationWindow,
    buffer: &impl IsA<gtk::TextBuffer>,
    editor_view: SourceView,
    file_state: FileState,
    panels: crate::ui::panels::notebook::PanelNotebooks,
    search_bar: SearchBar,
    search_entry: SearchEntry,
) -> gtk::gio::Menu {
    let lex_view = &panels.lexic_view;
    let errors_view = &panels.errors_view;
    let syntax_errors_view = &panels.syntax_errors_view;
    let semantic_errors_view = &panels.semantic_errors_view;
    let ast_view = &panels.ast_view;
    let editor_buffer: gtk::TextBuffer = buffer.as_ref().clone();

    ErrorNavigator::connect_error_click(&errors_view, &editor_buffer, &editor_view);
    ErrorNavigator::connect_error_click(&syntax_errors_view, &editor_buffer, &editor_view);
    ErrorNavigator::connect_error_click(&semantic_errors_view, &editor_buffer, &editor_view);
    LexicNavigator::connect_position_click(&lex_view, &editor_buffer, &editor_view);
    AstNavigator::connect_ast_click(&ast_view, &editor_buffer, &editor_view);

    AstNavigator::connect_ast_click(&panels.semantic_view, &editor_buffer, &editor_view);
    let symbol_buffer = editor_buffer.clone();
    let symbol_editor = editor_view.clone();
    panels
        .symbols_view
        .view
        .connect_activate(move |view, position| {
            if let Some(object) = view
                .model()
                .and_then(|m| m.item(position))
                .and_downcast::<gtk::glib::BoxedAnyObject>()
            {
                let symbol = object.borrow::<crate::compiler::semantic::Symbol>();
                ErrorNavigator::navigate_to(
                    &symbol_editor,
                    &symbol_buffer,
                    symbol.line,
                    symbol.column,
                );
            }
        });

    ActionHandlers::register_all(
        app,
        window,
        buffer,
        editor_view.clone(),
        file_state,
        panels,
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
