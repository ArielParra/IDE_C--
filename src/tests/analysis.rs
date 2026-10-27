use crate::compiler::analyze_source;

#[test]
fn source_pipeline_is_independent_of_saved_tokens() {
    let first = analyze_source("main { int x; x = 1; }");
    let second = analyze_source("main { int x; x = true; }");
    assert!(first.semantic.unwrap().errors.is_empty());
    assert!(!second.semantic.unwrap().errors.is_empty());
}

#[test]
fn source_pipeline_stops_on_lexical_errors_and_marks_partial_syntax() {
    let report = analyze_source("main { @ }");
    assert!(!report.lexical_errors.is_empty());
    assert!(report.ast.is_none());
    assert!(report.semantic.is_none());
    let report = analyze_source("main { int x; x = 1;");
    assert!(!report.syntax_errors.is_empty());
    assert!(report.ast.is_some());
    assert!(report.semantic.is_some());
}

#[test]
#[ignore = "requires a GTK display; run with xvfb-run and --test-threads=1"]
fn gtk_semantic_views_follow_current_buffer_and_invalidate() {
    use gtk::prelude::*;
    gtk::init().unwrap();
    let app = gtk::Application::builder()
        .application_id("com.ide_cmm.test")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let editor = sourceview5::View::new();
    let buffer = editor.buffer();
    let panels = crate::ui::panels::create_panels();
    let window = gtk::ApplicationWindow::new(&app);
    crate::ui::menu::build_menu(
        &app,
        &window,
        &buffer,
        editor.clone(),
        crate::models::new_file_state(),
        panels.clone(),
        gtk::SearchBar::new(),
        gtk::SearchEntry::new(),
    );
    let text = |view: &gtk::TextView| {
        let b = view.buffer();
        b.text(&b.start_iter(), &b.end_iter(), true).to_string()
    };
    buffer.set_text("main {\n float x;\n x = 2 + 3;\n}");
    app.activate_action("semantic", None);
    assert!(
        panels
            .semantic_view
            .borrow()
            .list_view
            .model()
            .unwrap()
            .n_items()
            > 0
    );
    assert_eq!(panels.symbols_view.view.model().unwrap().n_items(), 1);
    assert!(text(&panels.semantic_errors_view.borrow()).contains("No semantic errors"));
    fn copy_button(widget: &gtk::Widget, tooltip: &str) -> Option<gtk::Button> {
        if let Some(button) = widget.downcast_ref::<gtk::Button>() {
            if button.tooltip_text().as_deref() == Some(tooltip) {
                return Some(button.clone());
            }
        }
        let mut child = widget.first_child();
        while let Some(widget) = child {
            if let Some(button) = copy_button(&widget, tooltip) {
                return Some(button);
            }
            child = widget.next_sibling();
        }
        None
    }
    for (index, tooltip, expected) in [
        (1, "Copy Syntax Tree", "AddOp(+)"),
        (2, "Copy Annotated Tree", "value: 5"),
        (3, "Copy Symbol Table", "Offset (slot)"),
    ] {
        let page = panels.debug.nth_page(Some(index)).unwrap();
        let button = copy_button(&page, tooltip).expect(tooltip);
        button.emit_clicked();
        let copied = gtk::glib::MainContext::default()
            .block_on(button.clipboard().read_text_future())
            .unwrap()
            .unwrap();
        assert!(copied.contains(expected), "{tooltip}: {copied}");
        if index == 1 {
            assert!(!copied.contains("[type:"));
        }
    }
    // Activate the actual table/tree navigation signals and verify editor positions.
    panels
        .symbols_view
        .view
        .emit_by_name::<()>("activate", &[&0u32]);
    let cursor = buffer.iter_at_mark(&buffer.get_insert());
    assert_eq!(cursor.line(), 1);
    let model = panels.semantic_view.borrow().list_view.model().unwrap();
    let mut navigated = false;
    for index in 0..model.n_items() {
        let row = model
            .item(index)
            .and_downcast::<gtk::TreeListRow>()
            .unwrap();
        let node = row
            .item()
            .and_downcast::<crate::ui::panels::ast_view::AstNodeObject>()
            .unwrap();
        if node.line() == 3 && node.label().starts_with("AddOp(+)") {
            assert!(node.label().contains("value: 5"));
            assert!(node.label().contains("int -> float"));
            panels
                .semantic_view
                .borrow()
                .list_view
                .emit_by_name::<()>("activate", &[&index]);
            let cursor = buffer.iter_at_mark(&buffer.get_insert());
            assert_eq!(cursor.line(), 2);
            assert_eq!(cursor.line_offset() as u32, node.column() - 1);
            navigated = true;
            break;
        }
    }
    assert!(navigated, "Missing navigable annotated expression");
    assert!(
        !panels
            .ast_view
            .borrow()
            .list_view
            .model()
            .unwrap()
            .item(0)
            .and_downcast::<gtk::TreeListRow>()
            .unwrap()
            .item()
            .and_downcast::<crate::ui::panels::ast_view::AstNodeObject>()
            .unwrap()
            .label()
            .contains("type:")
    );
    buffer.insert_at_cursor(" ");
    assert_eq!(
        panels
            .semantic_view
            .borrow()
            .list_view
            .model()
            .unwrap()
            .n_items(),
        0
    );
    assert_eq!(panels.symbols_view.view.model().unwrap().n_items(), 0);
    assert!(text(&panels.semantic_errors_view.borrow()).is_empty());
    buffer.set_text("main { int x; x = true; }");
    app.activate_action("semantic", None);
    assert!(text(&panels.semantic_errors_view.borrow()).contains("Cannot assign"));
    buffer.set_text("main { int x; x = 1;");
    app.activate_action("semantic", None);
    let errors = text(&panels.semantic_errors_view.borrow());
    assert!(errors.contains("partial"));
    assert!(!errors.contains("No semantic errors"));
    // The standalone syntax action must also work on an unsaved buffer with no token file.
    buffer.set_text("main { float current; }");
    app.activate_action("semantic", None);
    app.activate_action("syntax", None);
    let syntax_model = panels.ast_view.borrow().list_view.model().unwrap();
    let labels: Vec<_> = (0..syntax_model.n_items())
        .map(|index| {
            syntax_model
                .item(index)
                .and_downcast::<gtk::TreeListRow>()
                .unwrap()
                .item()
                .and_downcast::<crate::ui::panels::ast_view::AstNodeObject>()
                .unwrap()
                .label()
                .to_string()
        })
        .collect();
    assert!(labels.iter().any(|label| label == "current"), "{labels:?}");
    buffer.set_text("main { @ }");
    app.activate_action("semantic", None);
    assert_eq!(panels.symbols_view.view.model().unwrap().n_items(), 0);
    assert_eq!(
        panels
            .semantic_view
            .borrow()
            .list_view
            .model()
            .unwrap()
            .n_items(),
        0
    );
    assert!(text(&panels.semantic_errors_view.borrow()).contains("not run"));
}
