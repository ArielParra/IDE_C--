use std::env;
use sourceview5::{LanguageManager, StyleSchemeManager};

// ================= LANGUAGE + STYLE =================

// Configura el lenguaje de programación y el esquema de colores del editor.
// Busca los archivos de lenguaje y estilo en recursos, carga el lenguaje "cmm" 
// y el tema "dark", y devuelve ambos.

pub fn create_language_and_style(
) -> (
    LanguageManager,
    StyleSchemeManager,
    Option<sourceview5::Language>,
    Option<sourceview5::StyleScheme>,
) {
    let base = env::current_dir().unwrap();

    let lang_path = base.join("src/resources/language-specs");
    let style_path = base.join("src/resources/styles");

    let manager = LanguageManager::new();

    let mut paths = manager.search_path();
    paths.push(lang_path.to_string_lossy().to_string().into());

    let paths: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    manager.set_search_path(&paths);

    let lang = manager.language("cmm");

    let sm = StyleSchemeManager::new();

    let mut paths = sm.search_path();
    paths.push(style_path.to_string_lossy().to_string().into());

    let paths: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    sm.set_search_path(&paths);
    println!("{:?}", lang_path);
    println!("{:?}", style_path);

    let scheme = sm.scheme("dark");

    (manager, sm, lang, scheme)
}