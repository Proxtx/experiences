#![feature(let_chains)]

use {
    api::experiences,
    rocket::{
        catch, catchers,
        fs::FileServer,
        response::{content, status},
        routes, Request,
    },
    tokio::{fs::File, io, sync::RwLock},
};

mod api;
mod config;
mod experience_manager;
mod renderer;

#[rocket::launch]
async fn rocket() -> _ {
    let config = config::Config::load()
        .await
        .unwrap_or_else(|e| panic!("Unable to init Config: {}", e));
    let experience_manager = experience_manager::ExperienceManager::new(&config).await;

    let figment = rocket::Config::figment().merge(("port", config.port));
    rocket::custom(figment)
        .register("/", catchers![not_found])
        .manage(config)
        .manage(experience_manager)
        .manage(api::navigator::NavigatorPosition(RwLock::new(
            "0".to_string(),
        )))
        .mount("/", FileServer::from("../frontend/dist/"))
        .mount(
            "/api",
            routes![
                experiences::create_experience,
                experiences::get_experience,
                experiences::favorite_event,
                experiences::delete_event,
                experiences::change_visibility,
                experiences::append_event,
                experiences::cover,
                experiences::entire_experience_cover,
                api::navigator::get_connections,
                api::navigator::get_position,
                api::timeline_url,
                api::auth_request
            ],
        )
}

#[catch(404)]
async fn not_found(
    _req: &Request<'_>,
) -> Result<status::Accepted<content::RawHtml<File>>, io::Error> {
    match File::open("../frontend/dist/index.html").await {
        Ok(v) => Ok(status::Accepted(content::RawHtml(v))),
        Err(e) => Err(e),
    }
}
