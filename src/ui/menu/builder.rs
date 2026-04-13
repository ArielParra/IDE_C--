use gtk::{gio, PopoverMenuBar};

pub struct MenuBuilder {
    model: gio::Menu,
}

impl MenuBuilder {
    pub fn new() -> Self {
        Self {
            model: gio::Menu::new(),
        }
    }

    pub fn build(&self) -> PopoverMenuBar {
        PopoverMenuBar::from_model(Some(&self.model))
    }

    pub fn add_file_menu(&mut self) -> &mut Self {
        let menu = gio::Menu::new();
        menu.append(Some("New"), Some("app.new"));
        menu.append(Some("Open"), Some("app.open"));
        menu.append(Some("Close"), Some("app.close"));
        menu.append(Some("Save"), Some("app.save"));
        menu.append(Some("Save As"), Some("app.save_as"));
        menu.append(Some("Exit"), Some("app.exit"));
        self.model.append_submenu(Some("File"), &menu);
        self
    }

    pub fn add_edit_menu(&mut self) -> &mut Self {
        let menu = gio::Menu::new();
        menu.append(Some("Undo"), Some("app.undo"));
        menu.append(Some("Redo"), Some("app.redo"));
        menu.append(Some("Cut"), Some("app.cut"));
        menu.append(Some("Copy"), Some("app.copy"));
        self.model.append_submenu(Some("Edit"), &menu);
        self
    }

    pub fn add_build_menu(&mut self) -> &mut Self {
        let menu = gio::Menu::new();
        menu.append(Some("Compile"), Some("app.compile1"));
        menu.append(Some("Run"), Some("app.run"));
        menu.append(Some("Debug"), Some("app.debug"));
        self.model.append_submenu(Some("Build & Debug"), &menu);
        self
    }

    pub fn add_analysis_menu(&mut self) -> &mut Self {
        let lexical = gio::Menu::new();
        lexical.append(Some("Run Lexical Analysis"), Some("app.lexical"));

        let syntax = gio::Menu::new();
        syntax.append(Some("Run Syntax Analysis"), Some("app.syntax"));

        let semantic = gio::Menu::new();
        semantic.append(Some("Run Semantic Analysis"), Some("app.semantic"));

        let compiler = gio::Menu::new();
        compiler.append(Some("Compile"), Some("app.c--compiler"));
        compiler.append(Some("Lexical Analysis"), Some("app.lexico"));
        compiler.append(Some("Syntax Analysis"), Some("app.sintactico"));
        compiler.append(Some("Semantic Analysis"), Some("app.semantico"));
        compiler.append(Some("Intermediate Code"), Some("app.intermedio"));
        compiler.append(Some("Execute"), Some("app.ejecutar"));

        self.model.append_submenu(Some("Lexical Analysis"), &lexical);
        self.model.append_submenu(Some("Syntax Analysis"), &syntax);
        self.model.append_submenu(Some("Semantic Analysis"), &semantic);
        self.model.append_submenu(Some("Compile"), &compiler);
        self
    }
}

impl Default for MenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}
