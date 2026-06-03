mod compiler;
mod file_manager;
mod models;
mod ui;

#[cfg(test)]
mod tests;

use gtk::gdk::Display;
use gtk::gio;
use gtk::{Application, IconTheme};
use gtk::{prelude::*, CssProvider};

fn main() {
    gio::resources_register_include!("compiled.gresource").expect("Failed to register resources.");

    let app = Application::builder()
        .application_id("com.ide_cmm.ide")
        .build();

    app.connect_startup(|app| {
        load_css();
        setup_icons();
        apply_system_theme();

        // Standard IDE keyboard shortcuts
        app.set_accels_for_action("app.new", &["<Primary>n"]);
        app.set_accels_for_action("app.open", &["<Primary>o"]);
        app.set_accels_for_action("app.save", &["<Primary>s"]);
        app.set_accels_for_action("app.save_as", &["<Primary><Shift>s"]);
        app.set_accels_for_action("app.close", &["<Primary>w"]);
        app.set_accels_for_action("app.exit", &["<Primary>q"]);
        app.set_accels_for_action("app.c--compiler", &["<Primary>r"]);
        
        // Edit menu shortcuts
        app.set_accels_for_action("app.undo", &["<Primary>z"]);
        app.set_accels_for_action("app.redo", &["<Primary>y", "<Primary><Shift>z"]);
        app.set_accels_for_action("app.find", &["<Primary>f"]);
        app.set_accels_for_action("app.cut", &["<Primary>x"]);
        app.set_accels_for_action("app.copy", &["<Primary>c"]);
    });
    app.connect_activate(|app| {
        let window = ui::Window::build(&app);
        window.present();
    });

    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_resource("/com/ide_cmm/ide/styles.css");

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

fn setup_icons() {
    if let Some(display) = Display::default() {
        let icon_theme = IconTheme::for_display(&display);
        icon_theme.add_resource_path("/com/ide_cmm/ide");
        gtk::Window::set_default_icon_name("com.ide_cmm.ide");
    }
}
