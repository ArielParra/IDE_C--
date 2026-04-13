mod compiler;
mod file_manager;
mod models;
mod ui;

use gtk::gdk::Display;
use gtk::gio;
use gtk::Application;
use gtk::{prelude::*, CssProvider};

fn main() {
    let app = Application::builder()
        .application_id("com.ide_cmm.ide")
        .build();

    app.connect_startup(|_| {
        load_css();
        apply_system_theme();
    });
    app.connect_activate(|app| {
        let window = ui::Window::build(&app);
        window.present();
    });

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

fn apply_system_theme() {
    if let Some(display) = Display::default() {
        let settings = gtk::Settings::for_display(&display);

        #[cfg(windows)]
        {
            use winreg::enums::*;
            use winreg::RegKey;

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(personalize) = hkcu
                .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
            {
                if let Ok(apps_use_light_theme) =
                    personalize.get_value::<u32, _>("AppsUseLightTheme")
                {
                    settings.set_property(
                        "gtk-application-prefer-dark-theme",
                        apps_use_light_theme == 0,
                    );
                }
            }
        }

        #[cfg(not(windows))]
        {
            settings.set_property(
                "gtk-application-prefer-dark-theme",
                settings.is_gtk_application_prefer_dark_theme(),
            );
        }
    }
}
