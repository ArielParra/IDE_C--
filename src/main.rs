use gtk::{CssProvider, prelude::*};
use gtk::{
    Box, Orientation,
    TextView, ScrolledWindow,Paned, gio, Notebook, Label
};
    use gtk::ApplicationWindow;
    use gtk::Application;

use gtk::gdk::Display;
use sourceview5::{View, Buffer};
use sourceview5::prelude::ViewExt;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::{HeaderBar, Button};

mod file_manager;
mod ui;

fn main() {

    let app = Application::builder()
        .application_id("com.ide_cmm.ide")
        .build();
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);
    //app.connect_activate(build_ui);
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

    // === Editor ===
    let buffer = Buffer::new(None);
    let source_view = View::with_buffer(&buffer);
    source_view.set_show_line_numbers(true);
    source_view.set_highlight_current_line(true);
    source_view.set_monospace(true);

    // NOTEBOOKS
    let debugnotebook = Notebook::new();
    debugnotebook.add_css_class("nopadding");
    let errorsnotebook = Notebook::new();
    
    // FOR DONDE CREAREMOS LABEL, SCROLLED Y SOURCEVIEW
    let labels_debug = ["Lexic","Syntax","Semantic","Hash Table","Intermediate Code"];
    for i in 0..5{  
        let textview = TextView::new();
        textview.set_editable(false);
        let scrolled = ScrolledWindow::builder()
        .child(&textview)
        .vexpand(true)
        .hexpand(true)
        .build();
        let label = Label::new(Some(&labels_debug[i]));
        debugnotebook.append_page(&scrolled, Some(&label));

    }
     let labels_errors = ["Lexic Errors","Syntax Errors","Semantic Errors","Results"];
    for i in 0..4{  
        let textview = TextView::new();
        textview.set_editable(false);
        let scrolled = ScrolledWindow::builder()
        .child(&textview)
        .vexpand(true)
        .hexpand(true)
        .build();
        let label = Label::new(Some(&labels_errors[i]));
        errorsnotebook.append_page(&scrolled, Some(&label));
    }


    // file state
    let file_state: Rc<RefCell<Option<PathBuf>>> =
        Rc::new(RefCell::new(None));

    // INSTANCIAS DE LOS SCROLLED WINDOWS
    let codigo = ScrolledWindow::builder()
        .child(&source_view)
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
    errorbox.append(&errorsnotebook);


    // AGREGAMOS CAJA DEL CODIGO Y CAJA DEL DEBUG AL PANEL SUPERIOR
    panedtop.set_start_child(Some(&principalcodebox));
    panedtop.set_end_child(Some(&debugbox));

    // AGREGAMOS EL PANEL SUPERIOR Y LA CAJA DE ERRORES AL PANEL PRINCIPAL
    panedprincipal.set_start_child(Some(&panedtop));
    panedprincipal.set_end_child(Some(&errorbox));

  
        
    // === Menu ===
    let menubar =
    ui::menu::build_menu(app, &window, &buffer, file_state.clone());



    // ===== HEADERBAR =====
    let header = HeaderBar::new();

    // ------ NEW FILE ICON ------
    let new_btn = Button::builder()
        .icon_name("document-new-symbolic")
        .tooltip_text("New File")
        .build();

    let app_clone = app.clone();
    new_btn.connect_clicked(move |_| {
        app_clone.activate_action("new", None);
    });

    // ------ OPEN FILE ICON ------
    let open_btn = Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Open File")
        .build();

    let app_clone = app.clone();
    open_btn.connect_clicked(move |_| {
        app_clone.activate_action("open", None);
    });

    // ------ CLOSE FILE ICON ------
     let close_btn = Button::builder()
        .icon_name("document-close-symbolic")
        .tooltip_text("Close File")
        .build();  

    let app_clone = app.clone();
    close_btn.connect_clicked(move |_| {
        app_clone.activate_action("close", None);
    });

    // ------ SAVE FILE ICON ------ 
    let save_btn = Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Save File")
        .build();

    let app_clone = app.clone();
    save_btn.connect_clicked(move |_| {
        app_clone.activate_action("save", None);
    });

    // ------ SAVE AS FILE ICON ------
    let save_as_btn = Button::builder()
        .icon_name("document-save-as-symbolic")
        .tooltip_text("Save As File")
        .build();

    let app_clone = app.clone();
    save_as_btn.connect_clicked(move |_| {
        app_clone.activate_action("save_as", None);
    });

    // ------ EXIT ICON ------
    let exit_btn = Button::builder()
        .icon_name("application-exit-symbolic")
        .tooltip_text("Exit")
        .build();

    let app_clone = app.clone();
    exit_btn.connect_clicked(move |_| {
        app_clone.activate_action("exit", None);
    });

    // ------ EXECUTE ICON ------
    let execute_btn = Button::builder()
        .icon_name("system-run-symbolic")
        .tooltip_text("Execute")
        .build();

    let app_clone = app.clone();
    execute_btn.connect_clicked(move |_| {
        app_clone.activate_action("execute", None);
    });

    // Opcional: estilo plano moderno
    open_btn.add_css_class("flat");
    new_btn.add_css_class("flat");
    close_btn.add_css_class("flat");
    save_btn.add_css_class("flat");
    save_as_btn.add_css_class("flat");
    exit_btn.add_css_class("flat");
    execute_btn.add_css_class("flat");

    // Add to header
     header.pack_start(&open_btn);
     header.pack_start(&new_btn);
     header.pack_start(&close_btn);
     header.pack_start(&save_btn);
     header.pack_start(&save_as_btn);
     header.pack_start(&exit_btn);
     header.pack_start(&execute_btn);

    // Asignamos header a la ventana
    window.set_titlebar(Some(&header));

    // AGREGAMOS MENUBAR Y PANEL PRINCIPAL A LA CAJA DE LA VENTANA PRINCIPAL
    windowbox.append(&menubar);
    windowbox.append(&panedprincipal);

    
    // BOX DE LINEAS Y COLUMNAS
    let bottomline = Box::new(Orientation::Vertical, 0);
    bottomline.set_size_request(0, 20);

    let textocollines = Rc::new(Label::new(None));
    let textocollines_clone = Rc::clone(&textocollines);

    textocollines.set_text("Lin 1, Col 1");
    textocollines.set_xalign(0.0);


    buffer.connect_cursor_position_notify(move |buffer| {
        let cursor_position = buffer.cursor_position();
        let iter = buffer.iter_at_offset(cursor_position);

        let line = iter.line();
        let col = iter.line_offset();

        let text = format!("Lin {}, Col {}", line + 1, col + 1);
        textocollines_clone.set_text(&text);
    });

    bottomline.append(&*textocollines);
    bottomline.set_margin_start(10);


    windowbox.append(&bottomline);

    // AGREGAMOS LA CAJA PRINCIPAL A LA VENTANA 
    window.set_child(Some(&windowbox));

    window.present();
    
}

