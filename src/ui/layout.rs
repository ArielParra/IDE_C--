// ================= LAYOUT =================
use gtk::prelude::*;
use gtk::{Box,Notebook, Orientation, Paned, ScrolledWindow};

// ================= LAYOUT =================

// Organiza visualmente los componentes del IDE en paneles.
// Coloca el editor a la izquierda, debug a la derecha y errores abajo en un Paned vertical principal.


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



