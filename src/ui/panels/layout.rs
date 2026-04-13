use gtk::prelude::*;
use gtk::{Box, Notebook, Orientation, Paned, ScrolledWindow};

pub struct Layout {
    pub container: Paned,
}

impl Layout {
    pub fn new(
        editor: &ScrolledWindow,
        debug_panel: &Notebook,
        errors_panel: &Notebook,
    ) -> Self {
        let panedtop = Paned::new(Orientation::Horizontal);
        panedtop.set_position(500);

        let editor_box = Box::new(Orientation::Vertical, 0);
        editor_box.append(editor);

        let debug_box = Box::new(Orientation::Vertical, 0);
        debug_box.append(debug_panel);

        panedtop.set_start_child(Some(&editor_box));
        panedtop.set_end_child(Some(&debug_box));

        let error_box = Box::new(Orientation::Vertical, 0);
        error_box.append(errors_panel);

        let main_paned = Paned::new(Orientation::Vertical);
        main_paned.set_start_child(Some(&panedtop));
        main_paned.set_end_child(Some(&error_box));

        Layout { container: main_paned }
    }
}
