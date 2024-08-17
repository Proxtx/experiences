pub use {
    rocket::get,
    rocket::http::{CookieJar, Status},
    rocket::post,
    rocket::response::status,
    rocket::serde::json::Json,
    rocket::State,
    serde::{Deserialize, Serialize},
    shared::timeline::types::api::{APIError, APIResult, AvailablePlugins, CompressedEvent},
    shared::types::Experience,
    shared::types::{
        ExperienceConnection, ExperienceConnectionResponse, ExperienceError, ExperienceEvent,
    },
    tokio::sync::RwLock,
};

use {crate::config::Config, rocket::response::Redirect, std::path::PathBuf};

pub mod experiences {
    use super::*;
    use crate::config::Config;
    use crate::experience_manager::ExperienceManager;

    #[post("/experience/<id>")]
    pub async fn get_experience(
        id: &str,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Json<APIResult<Experience>>> {
        let experience = match experience_manager.get_experience(id).await {
            Ok(v) => v,
            Err(e) => {
                return match &e {
                    ExperienceError::NotFound(_) => {
                        status::Custom(Status::NotFound, Json(Err(e.into())))
                    }
                    _ => status::Custom(Status::InternalServerError, Json(Err(e.into()))),
                }
            }
        };

        if experience.public {
            status::Custom(Status::Accepted, Json(Ok(experience)))
        } else {
            match auth(cookies, config) {
                Ok(_) => status::Custom(Status::Accepted, Json(Ok(experience))),
                Err(e) => status::Custom(Status::Unauthorized, Json(Err(e))),
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct CreateExperienceRequest {
        name: String,
    }

    #[post("/experience/create", data = "<request>")]
    pub async fn create_experience(
        request: Json<CreateExperienceRequest>,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Json<APIResult<String>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }

        match experience_manager
            .create_experience(request.name.clone())
            .await
        {
            Ok(v) => status::Custom(Status::Ok, Json(Ok(v))),
            Err(e) => status::Custom(Status::InternalServerError, Json(Err(e.into()))),
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct FavoriteRequest {
        event_id: String,
        favorite: bool,
    }

    #[post("/experience/<id>/favorite", data = "<request>")]
    pub async fn favorite_event(
        id: &str,
        request: Json<FavoriteRequest>,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Json<APIResult<()>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }

        match experience_manager
            .favorite_event(id, &request.event_id, request.favorite)
            .await
        {
            Ok(_) => status::Custom(Status::Ok, Json(Ok(()))),
            Err(e) => match &e {
                ExperienceError::NotFound(_) => {
                    status::Custom(Status::NotFound, Json(Err(e.into())))
                }
                _ => status::Custom(Status::InternalServerError, Json(Err(e.into()))),
            },
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct DeleteRequest {
        event_id: String,
    }

    #[post("/experience/<id>/delete", data = "<request>")]
    pub async fn delete_event(
        id: &str,
        request: Json<DeleteRequest>,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Json<APIResult<Option<(AvailablePlugins, ExperienceEvent)>>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }

        match experience_manager.delete_event(id, &request.event_id).await {
            Ok(v) => status::Custom(Status::Ok, Json(Ok(v))),
            Err(e) => match &e {
                ExperienceError::NotFound(_) => {
                    status::Custom(Status::NotFound, Json(Err(e.into())))
                }
                _ => status::Custom(Status::InternalServerError, Json(Err(e.into()))),
            },
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct VisibilityRequest {
        visibility: bool,
    }

    #[post("/experience/<id>/visibility", data = "<request>")]
    pub async fn change_visibility(
        id: &str,
        request: Json<VisibilityRequest>,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Json<APIResult<()>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }

        match experience_manager
            .set_experience_visibility(id, request.visibility)
            .await
        {
            Ok(_) => status::Custom(Status::Ok, Json(Ok(()))),
            Err(e) => match &e {
                ExperienceError::NotFound(_) => {
                    status::Custom(Status::NotFound, Json(Err(e.into())))
                }
                _ => status::Custom(Status::InternalServerError, Json(Err(e.into()))),
            },
        }
    }

    #[post("/experience/<id>/append_event", data = "<request>")]
    pub async fn append_event(
        id: &str,
        request: Json<(AvailablePlugins, CompressedEvent)>,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Json<APIResult<String>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }

        match experience_manager.append_event(id, request.0).await {
            Ok(v) => status::Custom(Status::Ok, Json(Ok(v))),
            Err(e) => match &e {
                ExperienceError::NotFound(_) => {
                    status::Custom(Status::NotFound, Json(Err(e.into())))
                }
                _ => status::Custom(Status::InternalServerError, Json(Err(e.into()))),
            },
        }
    }
}

pub mod navigator {
    use super::*;
    use crate::config::Config;
    use crate::experience_manager::ExperienceManager;

    pub struct NavigatorPosition(pub RwLock<String>);

    #[post("/navigator/<id>")]
    pub async fn get_connections(
        id: &str,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
        navigator_position: &State<NavigatorPosition>,
    ) -> status::Custom<Json<APIResult<ExperienceConnectionResponse>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }

        match experience_manager.get_experience(id).await {
            Ok(v) => {
                *navigator_position.0.write().await = id.to_string();
                let res = v
                    .events
                    .get(&AvailablePlugins::timeline_plugin_experience)
                    .map(|v| {
                        v.iter()
                            .map(|v| ExperienceConnection {
                                name: v.event.title.clone(),
                                id: v.id.clone(),
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or(Vec::new());
                status::Custom(
                    Status::Ok,
                    Json(Ok(ExperienceConnectionResponse {
                        connections: res,
                        experience_name: v.name.clone(),
                    })),
                )
            }
            Err(e) => match &e {
                ExperienceError::NotFound(_) => {
                    status::Custom(Status::NotFound, Json(Err(e.into())))
                }
                _ => status::Custom(Status::InternalServerError, Json(Err(e.into()))),
            },
        }
    }

    #[post("/navigator/position")]
    pub async fn get_position(
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        navigator_position: &State<NavigatorPosition>,
    ) -> status::Custom<Json<APIResult<String>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }
        status::Custom(
            Status::Ok,
            Json(Ok(navigator_position.0.read().await.clone())),
        )
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

#[post("/<path..>")]
pub fn api_redirect_post(config: &State<Config>, path: PathBuf) -> Redirect {
    Redirect::to(
        config
            .timeline_url
            .join("/api/")
            .unwrap()
            .join(path.to_str().unwrap_or(""))
            .unwrap()
            .to_string(),
    )
}

#[get("/<path..>")]
pub fn api_redirect_get(config: &State<Config>, path: PathBuf) -> Redirect {
    Redirect::to(
        config
            .timeline_url
            .join("/api/")
            .unwrap()
            .join(path.to_str().unwrap_or(""))
            .unwrap()
            .to_string(),
    )
}

#[post("/auth")]
pub fn auth_request(
    config: &State<Config>,
    cookies: &CookieJar<'_>,
) -> status::Custom<Json<APIResult<()>>> {
    status::Custom(Status::Ok, Json(auth(cookies, config)))
}
