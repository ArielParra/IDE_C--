fn main() {
    glib_build_tools::compile_resources(
        &["src"],
        "src/resources.gresource.xml",
        "compiled.gresource",
    );
}
