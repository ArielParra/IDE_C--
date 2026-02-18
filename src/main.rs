use gtk::glib::error;
use gtk::{CssProvider, prelude::*};
use gtk::{
    Application, ApplicationWindow, Box, Orientation,
    TextView, ScrolledWindow,Paned, gio, Notebook, Label
};
use gtk::gdk::Display;
use sourceview5::{View, Buffer};
use sourceview5::prelude::ViewExt;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

mod file_manager;
mod ui;

fn main() {
    let app = Application::builder()
        .application_id("com.ide_cmm.ide")
        .build();
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);
    app.run();
}
fn load_css(){
      // add css file
    let provider = CssProvider::new();

    let css = gio::File::for_path("src/styles.css");
    provider.load_from_file(&css);
      gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
 
}
fn build_ui(app: &Application) {
  
   let window = ApplicationWindow::builder()
        .application(app)
        .title("IDE_C--")
        .default_width(900)
        .default_height(600)
        .build();
   // let vbox = Box::new(Orientation::Vertical, 0);

    // TEXT VIEWS

    // === Editor ===
    let text_view = TextView::new();
    let buffer = text_view.buffer();

    // NOTEBOOKS
    let debugnotebook = Notebook::new();
    debugnotebook.add_css_class("nopadding");
    let erroresnotebook = Notebook::new();
    
    // FOR DONDE CREAREMOS LABEL, SCROLLED Y SOURCEVIEW
    let labels_debug = ["Lexico","Sintactico","Semantico","Hash Table","Codigo Intermedio"];
    for i in 0..4{  
        let textview = TextView::new();
        let scrolled = ScrolledWindow::builder()
        .child(&textview)
        .vexpand(true)
        .hexpand(true)
        .build();
        let label = Label::new(Some(&labels_debug[i]));
        debugnotebook.append_page(&scrolled, Some(&label));
    }
     let labels_errores = ["Errores Lexicos","Errores Sintacticos","Errores Semanticos","Resultados"];
    for i in 0..4{  
        let textview = TextView::new();
        let scrolled = ScrolledWindow::builder()
        .child(&textview)
        .vexpand(true)
        .hexpand(true)
        .build();
        let label = Label::new(Some(&labels_errores[i]));
        erroresnotebook.append_page(&scrolled, Some(&label));
    }


    // file state
    let file_state: Rc<RefCell<Option<PathBuf>>> =
        Rc::new(RefCell::new(None));

    // INSTANCIAS DE LOS SCROLLED WINDOWS
    let codigo = ScrolledWindow::builder()
        .child(&text_view)
        .vexpand(true)
        .hexpand(true)
        .build();
  
    // CAJA DE LA VENTANA QUE CONTIENE TODO
    let windowbox = Box::new(Orientation::Vertical,0);
    
    // PANEL SUPERIOR DE ARRIBA / CODIGO Y DEBUG
    let panedtop = Paned::new(Orientation::Horizontal);
    //panedtop.add_css_class("red");
    panedtop.set_vexpand(true);
    panedtop.set_hexpand(true);
    
    // NECESITA EXPANDIRSE UN POCO
    panedtop.set_position(500);
    

    // PANEL VERTICAL QUE CONTIENE EL PANEL SUPERIOR Y LA CAJA DE ERRORES
    let panedprincipal = Paned::new(Orientation::Vertical);
    panedprincipal.set_vexpand(true);
    panedprincipal.set_hexpand(true);

    // CAJA DONDE ESTARA EL CODIGO PRINCIPAL
    let principalcodebox =  Box::new(Orientation::Vertical, 0);
    //principalcodebox.add_css_class("red");
    principalcodebox.set_hexpand(true);
    principalcodebox.set_vexpand(true);
    principalcodebox.append(&codigo);

    // CAJA DONDE ESTARA EL DEBUG 
    let debugbox  = Box::new(Orientation::Vertical, 0);
    debugbox.add_css_class("margin");
    debugbox.set_hexpand(true);
    debugbox.set_vexpand(true);
    debugbox.append(&debugnotebook);
    //debugbox.append(&debug);

    // CAJA DONDE SE VERAN LOS ERRORES
    let errorbox = Box::new(Orientation::Vertical, 0);
    errorbox.add_css_class("margin");
    errorbox.set_hexpand(true);
    errorbox.set_vexpand(true);
    errorbox.append(&erroresnotebook);


    // AGREGAMOS CAJA DEL CODIGO Y CAJA DEL DEBUG AL PANEL SUPERIOR
    panedtop.set_start_child(Some(&principalcodebox));
    panedtop.set_end_child(Some(&debugbox));

    // AGREGAMOS EL PANEL SUPERIOR Y LA CAJA DE ERRORES AL PANEL PRINCIPAL
    panedprincipal.set_start_child(Some(&panedtop));
    panedprincipal.set_end_child(Some(&errorbox));

    // LA CAJA DE ERRORES DEBE VERSE MAS ABAJO 
    panedprincipal.set_position(700);

        
    // === Menu ===
    let menubar =
    ui::menu::build_menu(app, &window, &buffer, file_state.clone());

    // AGREGAMOS MENUBAR Y PANEL PRINCIPAL A LA CAJA DE LA VENTANA PRINCIPAL
    windowbox.append(&menubar);
    windowbox.append(&panedprincipal);
    // AGREGAMOS LA CAJA PRINCIPAL A LA VENTANA 
    window.set_child(Some(&windowbox));
    window.present();
    
}
