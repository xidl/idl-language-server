fn main() {
    println!("cargo:rerun-if-changed=assets/scalar.html");
    println!("cargo:rerun-if-changed=assets/api-reference.js");
    build_scalar_standalone();
}

fn build_scalar_standalone() {
    const SCALAR_RAW_HTML: &str = include_str!("./assets/scalar.html");
    const SCALAR_JS: &str = include_str!("./assets/api-reference.js");

    let scalar_html = SCALAR_RAW_HTML.replace("{{ api-reference }}", SCALAR_JS);
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let scalar_html_path = std::path::Path::new(&out_dir).join("scalar.standalone.html");

    std::fs::write(scalar_html_path, scalar_html).expect("write scalar html");
}
