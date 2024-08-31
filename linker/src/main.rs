use std::path::PathBuf;

use tokio::{
    fs::{write, File},
    io::AsyncReadExt,
};

#[tokio::main]
async fn main() {
    let mut file = File::open("main.Cargo.toml")
        .await
        .expect("Did not find preset cargo file");
    let mut str = String::new();
    file.read_to_string(&mut str)
        .await
        .expect("Unable to read preset cargo file to string");
    let mut timeline_location = String::new();
    let mut timeline_location_file = File::open("../timeline_location.txt")
        .await
        .expect("Did not find timeline location file!");
    timeline_location_file
        .read_to_string(&mut timeline_location)
        .await
        .expect("Unable to read timeline location file!");

    let timeline_directory = PathBuf::from("../").join(PathBuf::from(timeline_location));

    str += &format!(
        "\ntypes = {{path = \"{}\", features=[\"experiences\"]}}
        timeline_frontend = {{path = \"{}\", features=[\"experiences\"], optional=true}}
        \n",
        timeline_directory.join("types").display(),
        timeline_directory.join("frontend").display()
    );

    write("../shared/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");

    //navigator

    let mut navigator_file = File::open("navigator.Cargo.toml")
        .await
        .expect("Did not find preset cargo file");
    let mut str = String::new();
    navigator_file
        .read_to_string(&mut str)
        .await
        .expect("Unable to read preset cargo file to string");

    str += &format!(
        "\ntypes = {{path = \"{}\", features=[\"experiences\"]}}
        \n",
        timeline_directory.join("types").display(),
    );

    write("../experiences_navigator/Cargo.toml", str)
        .await
        .expect("Unable to write new Cargo.toml file");
}
