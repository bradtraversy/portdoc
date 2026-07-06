fn main() {
    // rust-embed requires web/dist to exist at compile time; a fresh clone
    // hasn't run the web build yet, so guarantee the folder.
    std::fs::create_dir_all("web/dist").expect("failed to create web/dist");
    println!("cargo:rerun-if-changed=web/dist");
}
