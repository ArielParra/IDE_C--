use gtk::prelude::*;
use gtk::{Application, Button, HeaderBar};

pub struct IDEHeaderBar {
    pub widget: HeaderBar,
}

impl IDEHeaderBar {
    pub fn new(_app: &Application) -> Self {
        let header = HeaderBar::new();

        let buttons = [
            ("document-new-symbolic", "app.new", "New (Ctrl+N)", "<Primary>n"),
            ("document-open-symbolic", "app.open", "Open (Ctrl+O)", "<Primary>o"),
            ("document-close-symbolic", "app.close", "Close (Ctrl+W)", "<Primary>w"),
            ("document-save-symbolic", "app.save", "Save (Ctrl+S)", "<Primary>s"),
            ("document-save-as-symbolic", "app.save_as", "Save As (Ctrl+Shift+S)", "<Primary><Shift>s"),
            ("edit-find-symbolic", "app.find", "Find (Ctrl+F)", "<Primary>f"),
            ("application-exit-symbolic", "app.exit", "Exit (Ctrl+Q)", "<Primary>q"),
            ("system-run-symbolic", "app.c--compiler", "Execute (Ctrl+R)", "<Primary>r"),
        ];

        for (icon, action, tooltip, shortcut_str) in buttons {
            let btn = Button::builder()
                .icon_name(icon)
                .action_name(action)
                .tooltip_text(tooltip)
                .build();

            // Add shortcut controller to visually activate the button
            if let Some(trigger) = gtk::ShortcutTrigger::parse_string(shortcut_str) {
                let shortcut = gtk::Shortcut::new(
                    Some(trigger),
                    Some(gtk::ShortcutAction::parse_string("activate").unwrap())
                );
                let controller = gtk::ShortcutController::new();
                controller.set_scope(gtk::ShortcutScope::Global);
                controller.add_shortcut(shortcut);
                btn.add_controller(controller);
            }

            btn.add_css_class("flat");
            header.pack_start(&btn);
        }

        IDEHeaderBar { widget: header }
    }
}
