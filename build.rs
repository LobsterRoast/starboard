fn main() {
    println!("cargo::rerun-if-changed=src/c/resources.c");

    let gio = pkg_config::probe_library("gio-2.0").expect("Couldn't find gio-2.0 library");

    cc::Build::new()
        .file("src/c/resources.c")
        .includes(gio.include_paths)
        .compile("resources");
}
