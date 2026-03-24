use gtk::{CssProvider, prelude::*};
use gtk::{gio};
use gtk::Application;
use gtk::gdk::Display;

mod file_manager;
mod compiler;
mod ui; 


fn main() {

    let app = Application::builder()
        .application_id("com.ide_cmm.ide")
        .build();

    app.connect_startup(|_| load_css());

    app.connect_activate(|app| ui::window::build_ui(app));

    app.run();
}

fn load_css() {
    let provider = CssProvider::new();

    let css = gio::File::for_path("src/styles.css");
    provider.load_from_file(&css);

    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}