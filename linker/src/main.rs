use std::path::PathBuf;

use tokio::{
    fs::{read_dir, write, File},
    io::AsyncReadExt,
};

#[tokio::main]
async fn main() {
    let mut timeline_location = String::new();
    let mut timeline_location_file = File::open("../timeline_location.txt")
        .await
        .expect("Did not find timeline location file!");
    timeline_location_file
        .read_to_string(&mut timeline_location)
        .await
        .expect("Unable to read timeline location file!");

    let timeline_directory = PathBuf::from("../").join(PathBuf::from(timeline_location));

    //timeline_types

    let mut timeline_types_file = File::open("timeline_types.Cargo.toml")
        .await
        .expect("Did not find preset cargo file");
    let mut str = String::new();
    timeline_types_file
        .read_to_string(&mut str)
        .await
        .expect("Unable to read preset cargo file to string");

    str += &format!(
        "\ntypes = {{path = \"{}\", features=[\"experiences\", \"client\"]}}
        \n",
        timeline_directory.join("types").display(),
    );

    write("../timeline_types/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");

    //link
    let mut types_file = File::open("link.Cargo.toml")
        .await
        .expect("Did not find preset cargo file");
    let mut str = String::new();
    types_file
        .read_to_string(&mut str)
        .await
        .expect("Unable to read preset cargo file to string");

    let mut dirs = read_dir("../plugins/")
        .await
        .expect("Unable to find plugins directory");

    let mut plugins_str = String::new();
    let mut server_features_str = String::new();

    while let Some(entry) = dirs
        .next_entry()
        .await
        .expect("Unable to read plugins directory")
    {
        let plugin_name = entry.file_name().into_string().expect("Unable to convert filename to string");
        server_features_str.push_str(&format!("\"dep:{}\", ", plugin_name));
        plugins_str.push_str(&format!("{0} = {{path=\"../plugins/{0}/\", optional=true}}\n", plugin_name));
    }

    str += &format!("server = [{} \"dep:server_api\"]\n[dependencies]\n", server_features_str);

    str += &plugins_str;

    str += &format!(
        "\ntimeline_frontend = {{path = \"{}\", features=[\"experiences\"], optional=true}}\n
        server_api = {{path=\"../server_api\", optional=true}}\n
        experiences_link_proc_macro = {{path=\"../experiences_link_proc_macro\"}}",
        timeline_directory.join("frontend").display(),
    );

    write("../link/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");
}
