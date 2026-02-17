use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

fn main() {
    let app = Application::builder()
        .application_id("com.ejemplo.MiApp")
        .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Mi primera app GTK en Rust")
            .default_width(400)
            .default_height(200)
            .build();

        window.present();
    });

    app.run();
}
