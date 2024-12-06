use std::path::PathBuf;

use tokio::{
    fs::{write, File},
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

    str += &format!(
        "\ntimeline_frontend_lib = {{path = \"{}\", features=[\"experiences\"], optional=true}}\n",
        timeline_directory.join("frontend").display(),
    );

    write("../link/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");
}
