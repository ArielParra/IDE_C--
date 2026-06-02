use gtk::prelude::*;
use gtk::TextView;
use sourceview5::View as SourceView;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ErrorNavigator;

impl ErrorNavigator {
    pub fn connect_error_click(
        errors_view: &Rc<RefCell<TextView>>,
        editor_buffer: &gtk::TextBuffer,
        editor_view: &SourceView,
    ) {
        let errors_textview = errors_view.borrow().clone();
        let errors_buffer = errors_textview.buffer();
        let editor_buffer_clone = editor_buffer.clone();
        let editor_view_clone = editor_view.clone();

        errors_buffer.connect_mark_set(move |buf, iter, mark| {
            if mark.name().as_deref() != Some("insert") {
                return;
            }

            let mut start = *iter;
            start.set_line_offset(0);

            let mut end = start;
            end.forward_to_line_end();

            let line_text = buf.text(&start, &end, true).to_string();

            if let Some((line, column)) = Self::parse_position(&line_text) {
                Self::navigate_to(&editor_view_clone, &editor_buffer_clone, line, column);
            }
        });
    }

    fn parse_position(line: &str) -> Option<(usize, usize)> {
        let open = line.rfind('(')?;
        let close = line.rfind(')')?;

        if close <= open + 1 {
            return None;
        }

        let pos = &line[open + 1..close];
        let (line_s, col_s) = pos.split_once(':')?;

        let line_n = line_s.trim().parse::<usize>().ok()?;
        let col_n = col_s.trim().parse::<usize>().ok()?;

        if line_n == 0 || col_n == 0 {
            return None;
        }

        Some((line_n, col_n))
    }

    fn navigate_to(
        editor_view: &SourceView,
        editor_buffer: &gtk::TextBuffer,
        line: usize,
        column: usize,
    ) {
        let line_idx = (line - 1) as i32;
        let col_idx = (column - 1) as i32;

        let mut iter = match editor_buffer.iter_at_line_offset(line_idx, col_idx) {
            Some(it) => it,
            None => match editor_buffer.iter_at_line(line_idx) {
                Some(it) => it,
                None => return,
            },
        };

        editor_buffer.place_cursor(&iter);

        let mut end = iter;
        if !end.ends_line() {
            end.forward_char();
        }
        editor_buffer.select_range(&iter, &end);

        editor_view.scroll_to_iter(&mut iter, 0.2, false, 0.0, 0.0);
        editor_view.grab_focus();
    }
}

pub struct LexicNavigator;

impl LexicNavigator {
    pub fn connect_position_click(
        lexic_view: &Rc<RefCell<TextView>>,
        editor_buffer: &gtk::TextBuffer,
        editor_view: &SourceView,
    ) {
        let lexic_view_clone = lexic_view.clone();
        let editor_buffer_clone = editor_buffer.clone();
        let editor_view_clone = editor_view.clone();
        let lexic_textview = lexic_view_clone.borrow().clone();
        let buffer = lexic_textview.buffer();

        buffer.connect_mark_set(move |buf, iter, mark| {
            if mark.name().as_deref() != Some("insert") {
                return;
            }

            let mut start = *iter;
            start.set_line_offset(0);

            let mut end = start;
            end.forward_to_line_end();

            let line_text = buf.text(&start, &end, true).to_string();

            if let Some((line, column)) = ErrorNavigator::parse_position(&line_text) {
                ErrorNavigator::navigate_to(&editor_view_clone, &editor_buffer_clone, line, column);
            }
        });
    }
}
