#![feature(let_chains)]

use std::{env, fmt::Write, fs, io::Read, path::PathBuf};
fn main() {
    println!("cargo:rerun-if-changed=../plugins/");
    println!("cargo:rerun-if-changed=build.rs");

    let mut timeline_location = String::new();
    let mut timeline_location_file =
        fs::File::open("../timeline_location.txt").expect("Did not find timeline location file!");
    timeline_location_file
        .read_to_string(&mut timeline_location)
        .expect("Unable to read timeline location file!");

    let timeline_directory = PathBuf::from("../").join(PathBuf::from(timeline_location));
    let plugins_directory = timeline_directory.join("plugins");

    let mut out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    out_path = out_path.join("out");
    out_path.set_file_name("plugins.rs");
    let mut plugins: Vec<(String, String)> = fs::read_dir(plugins_directory)
        .expect("Plugins Folder not found.")
        .filter_map(|v| {
            let dir_entry = v.expect("unable to read directory");
            if dir_entry
                .file_type()
                .expect("unable to read file-type")
                .is_file()
            {
                panic!("Did not expect a file in plugins folder");
            }
            let name = dir_entry
                .file_name()
                .into_string()
                .expect("unable to parse filename");
            let mut path = dir_entry.path();
            path.push("experiences_renderer.rs");
            if let Ok(exists) = fs::exists(&path)
                && exists
            {
                Some((
                    name,
                    fs::canonicalize(&path)
                        .expect("unable to resolve path")
                        .into_os_string()
                        .into_string()
                        .expect("os string error"),
                ))
            } else {
                None
            }
        })
        .collect();
    let mod_str = plugins.iter().fold(String::new(), |mut output, b| {
        let _ = write!(
            output,
            "
        #[path = \"{}\"]
        mod {};",
            b.1.replace('\\', "\\\\").replace('\"', "\\\""),
            b.0
        );
        output
    });
    let init_str = plugins
        .iter()
        .map(|v| {
            format!(
                "(AvailablePlugins::{}, Box::new({}::PluginRenderer::new().await) as Box<dyn PluginRenderer>)",
                v.0, v.0
            )
        })
        .collect::<Vec<String>>()
        .join(", ");
    let importer = format!(
        "
    //dynamic module imports
    {}
    pub struct PluginRenderers<'a> {{
        pub renderers: HashMap<AvailablePlugins, Box<dyn PluginRenderer + 'a>>,
    }}

    impl<'a> PluginRenderers<'a> {{
        pub async fn init() -> PluginRenderers<'a> {{
            PluginRenderers {{
                renderers: HashMap::from([{}])
            }}
        }}
    }}
    ",
        mod_str, init_str
    );
    fs::write(out_path, importer).expect("Unable to write plugins file");
}
