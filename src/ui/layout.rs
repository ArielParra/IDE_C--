// ================= LAYOUT =================
use gtk::prelude::*;
use gtk::{Box,Notebook, Orientation, Paned, ScrolledWindow};

// ================= LAYOUT =================

// Organizes the IDE components visually in panels.
// Places the editor on the left, debug on the right and errors at the bottom in a main vertical Paned.


pub fn create_layout(
    codigo: &ScrolledWindow,
    debugnotebook: &Notebook,
    errorsnotebook: &Notebook,
) -> Paned {

    let panedtop = Paned::new(Orientation::Horizontal);
    panedtop.set_position(500);

    let principalcodebox = Box::new(Orientation::Vertical, 0);
    principalcodebox.append(codigo);

    let debugbox = Box::new(Orientation::Vertical, 0);
    debugbox.append(debugnotebook);

    panedtop.set_start_child(Some(&principalcodebox));
    panedtop.set_end_child(Some(&debugbox));


    let errorbox = Box::new(Orientation::Vertical, 0);
    errorbox.append(errorsnotebook);


    let panedprincipal = Paned::new(Orientation::Vertical);

    panedprincipal.set_start_child(Some(&panedtop));
    panedprincipal.set_end_child(Some(&errorbox));

    panedprincipal
}



