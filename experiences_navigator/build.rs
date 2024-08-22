use stylers::build;
use std::path::PathBuf;
use std::env;

fn main() {
    let style_path =
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("../../../../generated_experiences_navigator.css");
    build(Some(style_path.display().to_string()));
}
