use gtk::gio;

pub struct MenuBuilder {
    model: gio::Menu,
}

impl MenuBuilder {
    pub fn new() -> Self {
        Self {
            model: gio::Menu::new(),
        }
    }

    pub fn build(&self) -> gio::Menu {
        self.model.clone()
    }

    pub fn add_file_menu(&mut self) -> &mut Self {
        let menu = gio::Menu::new();
        menu.append(Some("_New"), Some("app.new"));
        menu.append(Some("_Open"), Some("app.open"));
        menu.append(Some("_Close"), Some("app.close"));
        menu.append(Some("_Save"), Some("app.save"));
        menu.append(Some("Save _As"), Some("app.save_as"));
        menu.append(Some("E_xit"), Some("app.exit"));
        self.model.append_submenu(Some("_File"), &menu);
        self
    }

    pub fn add_edit_menu(&mut self) -> &mut Self {
        let menu = gio::Menu::new();
        menu.append(Some("_Undo"), Some("app.undo"));
        menu.append(Some("_Redo"), Some("app.redo"));
        menu.append(Some("Cu_t"), Some("app.cut"));
        menu.append(Some("_Copy"), Some("app.copy"));
        self.model.append_submenu(Some("_Edit"), &menu);
        self
    }

    pub fn add_build_menu(&mut self) -> &mut Self {
        let menu = gio::Menu::new();
        menu.append(Some("_Compile"), Some("app.compile1"));
        menu.append(Some("_Run"), Some("app.run"));
        menu.append(Some("_Debug"), Some("app.debug"));
        self.model.append_submenu(Some("_Build & Debug"), &menu);
        self
    }

    pub fn add_analysis_menu(&mut self) -> &mut Self {
        let lexical = gio::Menu::new();
        lexical.append(Some("Run _Lexical Analysis"), Some("app.lexical"));

        let syntax = gio::Menu::new();
        syntax.append(Some("Run _Syntax Analysis"), Some("app.syntax"));

        let semantic = gio::Menu::new();
        semantic.append(Some("Run Se_mantic Analysis"), Some("app.semantic"));

        let compiler = gio::Menu::new();
        compiler.append(Some("_Compile"), Some("app.c--compiler"));
        compiler.append(Some("_Lexical Analysis"), Some("app.lexico"));
        compiler.append(Some("_Syntax Analysis"), Some("app.sintactico"));
        compiler.append(Some("Se_mantic Analysis"), Some("app.semantico"));
        compiler.append(Some("_Intermediate Code"), Some("app.intermedio"));
        compiler.append(Some("_Execute"), Some("app.ejecutar"));

        self.model
            .append_submenu(Some("_Lexical Analysis"), &lexical);
        self.model.append_submenu(Some("S_yntax Analysis"), &syntax);
        self.model
            .append_submenu(Some("Se_mantic Analysis"), &semantic);
        self.model.append_submenu(Some("_Compiler"), &compiler);
        self
    }
}

impl Default for MenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}
