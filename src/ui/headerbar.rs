// ================= HEADERBAR =================
use gtk::prelude::*;
use gtk::{Application, Button, HeaderBar};

// ================= HEADERBAR =================

// Crea la barra de herramientas superior con botones de acciones (nuevo, abrir, guardar, ejecutar, salir...).
// Conecta cada botón a la acción correspondiente de la aplicación.

pub fn create_headerbar(app: &Application) -> HeaderBar {

    let header = HeaderBar::new();

    let buttons = [
        ("document-new-symbolic", "new"),
        ("document-open-symbolic", "open"),
        ("document-close-symbolic", "close"),
        ("document-save-symbolic", "save"),
        ("document-save-as-symbolic", "save_as"),
        ("application-exit-symbolic", "exit"),
        ("system-run-symbolic", "execute"),
    ];

    for (icon, action) in buttons {

        let btn = Button::builder()
            .icon_name(icon)
            .build();

        let app_clone = app.clone();

        btn.connect_clicked(move |_| {
            app_clone.activate_action(action, None);
        });

        btn.add_css_class("flat");

        header.pack_start(&btn);
    }

    header
}
