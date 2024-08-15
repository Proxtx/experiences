#![feature(let_chains)]

mod config;
mod experience_manager;

#[tokio::main]
async fn main() {
    let config = config::Config::load()
        .await
        .unwrap_or_else(|e| panic!("Unable to init Config: {}", e));
    println!("Hello, world!");
}
