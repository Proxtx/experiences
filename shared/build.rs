/*use std::env;
use std::fs;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() {
    println!("cargo:rerun-if-changed=../plugins/");
    println!("cargo:rerun-if-changed=build.rs");
    let mut out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    out_path = out_path.join("out");
    out_path.set_file_name("timeline_dyn.rs");

    let mut timeline_location = String::new();
    let mut timeline_location_file = File::open("../timeline_location.txt")
        .await
        .expect("Did not find timeline location file!");
    timeline_location_file
        .read_to_string(&mut timeline_location)
        .await
        .expect("Unable to read timeline location file!");

    let timeline_directory = PathBuf::from("../").join(PathBuf::from(timeline_location));

    /*let file = format!("
    #[path = \"{}\"]
    mod api;

    ", );*/

    //fs::write(out_path, file).expect("Unable to write timeline_dyn file");
}
*/
fn main() {}
