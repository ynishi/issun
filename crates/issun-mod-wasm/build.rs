fn main() {
    // Rerun build if WIT files change
    println!("cargo:rerun-if-changed=wit/issun.wit");
}
