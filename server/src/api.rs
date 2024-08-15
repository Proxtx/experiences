pub use {
    rocket::get,
    rocket::http::{CookieJar, Status},
    rocket::post,
    rocket::response::status,
    rocket::serde::json::Json,
    rocket::State,
    shared::timeline::types::api::{APIError, APIResult},
    shared::types::Experience,
};

use crate::config::Config;

pub mod experiences {
    use super::*;
    use crate::config::Config;
    use crate::experience_manager::ExperienceManager;

    #[get("/experience/<id>")]
    pub async fn get_experience(
        id: &str,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Json<APIResult<Experience>>> {
        experience_manager.get_experience(id).await
    }
}

pub fn auth(cookies: &CookieJar<'_>, config: &State<Config>) -> APIResult<()> {
    match cookies.get("pwd") {
        Some(pwd) => {
            if pwd.value() != config.password {
                Err(APIError::AuthenticationError)
            } else {
                Ok(())
            }
        }
        None => Err(APIError::AuthenticationError),
    }
}

#[post("/auth")]
pub fn auth_request(
    config: &State<Config>,
    cookies: &CookieJar<'_>,
) -> status::Custom<Json<APIResult<()>>> {
    status::Custom(Status::Ok, Json(auth(cookies, config)))
}
