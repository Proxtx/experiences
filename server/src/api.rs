pub use {
    crate::config::Config,
    rocket::{
        get,
        http::{ContentType, CookieJar, Status},
        post,
        response::status,
        serde::json::Json,
        State,
    },
    timeline_types::api::{APIError, APIResult, CompressedEvent},
    timeline_types::available_plugins::AvailablePlugins,
    shared::{
        standalone_experience_types::types::{ExperienceConnection, ExperienceConnectionResponse},
        types::{
            CreateExperienceRequest, Experience, ExperienceError, ExperienceEvent, FavoriteRequest,
        },
    },
    tokio::{fs::File, sync::RwLock},
};

pub mod experiences {
    use {
        super::*,
        crate::{config::Config, experience_manager::ExperienceManager},
        image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder},
        std::{path::PathBuf, thread},
    };

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
            .create_experience(request.name.clone(), request.time.clone())
            .await
        {
            Ok(v) => status::Custom(Status::Ok, Json(Ok(v))),
            Err(e) => status::Custom(Status::InternalServerError, Json(Err(e.into()))),
        }
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

    #[post("/experience/<id>/delete", data = "<request>")]
    pub async fn delete_event(
        id: &str,
        request: Json<String>,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Json<APIResult<Option<(AvailablePlugins, ExperienceEvent)>>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }

        match experience_manager.delete_event(id, &request).await {
            Ok(v) => status::Custom(Status::Ok, Json(Ok(v))),
            Err(e) => match &e {
                ExperienceError::NotFound(_) => {
                    status::Custom(Status::NotFound, Json(Err(e.into())))
                }
                _ => status::Custom(Status::InternalServerError, Json(Err(e.into()))),
            },
        }
    }

    #[post("/experience/<id>/visibility", data = "<request>")]
    pub async fn change_visibility(
        id: &str,
        request: Json<bool>,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Json<APIResult<()>>> {
        if let Err(e) = auth(cookies, config) {
            return status::Custom(Status::Unauthorized, Json(Err(e)));
        }

        match experience_manager
            .set_experience_visibility(id, *request)
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

    #[get("/experience/<id>/cover/<size>/all")]
    pub async fn entire_experience_cover(
        id: &str,
        size: u32,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Option<(ContentType, Vec<u8>)>> {
        let experience = match experience_manager.get_experience(id).await {
            Ok(v) => v,
            Err(e) => {
                return match &e {
                    ExperienceError::NotFound(_) => status::Custom(Status::NotFound, None),
                    _ => status::Custom(Status::InternalServerError, None),
                }
            }
        };

        if !experience.public
            && let Err(_e) = auth(cookies, config)
        {
            return status::Custom(Status::Unauthorized, None);
        }

        let renderer = experience_manager.renderer.clone();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let image = thread::spawn(move || {
            let local = tokio::task::LocalSet::new();

            local.block_on(&rt, async move {
                tokio::task::spawn_local(async move {
                    let mut image: Vec<u8> = Vec::new();
                    let encoder = PngEncoder::new(&mut image);
                    let data = renderer
                        .render_entire_experience(&experience, size as i32)
                        .await
                        .into_vec();
                    let data = data
                        .into_iter()
                        .flat_map(|v| {
                            let bts = v.to_le_bytes();
                            let r = bts[2];
                            let g = bts[1];
                            let b = bts[0];
                            let a = bts[3];
                            vec![r, g, b, a]
                        })
                        .collect::<Vec<u8>>();
                    let _ = encoder.write_image(&data, size, size, ExtendedColorType::Rgba8);
                    image
                })
                .await
            })
        })
        .join();

        if let Ok(outer) = image
            && let Ok(inner) = outer
        {
            status::Custom(Status::Ok, Some((ContentType::PNG, inner)))
        } else {
            status::Custom(Status::InternalServerError, None)
        }
    }

    #[get("/experience/<id>/cover/<size>")]
    pub async fn cover(
        id: &str,
        size: &str,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
    ) -> status::Custom<Option<(ContentType, Result<File, std::io::Error>)>> {
        let experience = match experience_manager.get_experience(id).await {
            Ok(v) => v,
            Err(e) => {
                return match &e {
                    ExperienceError::NotFound(_) => status::Custom(Status::NotFound, None),
                    _ => status::Custom(Status::InternalServerError, None),
                }
            }
        };

        if !experience.public
            && let Err(_e) = auth(cookies, config)
        {
            return status::Custom(
                Status::Unauthorized,
                Some((
                    ContentType::PNG,
                    File::open(PathBuf::from("../frontend/icons/logo.png")).await,
                )),
            );
        }

        let mut path = config.covers_folder.clone();
        if size == "small" {
            path = path.join(format!("{}.small.png", id));
        } else {
            path = path.join(format!("{}.png", id));
        }

        status::Custom(Status::Ok, Some((ContentType::PNG, File::open(path).await)))
    }
}

pub mod navigator {
    use {
        super::*,
        crate::{config::Config, experience_manager::ExperienceManager},
    };

    pub struct NavigatorPosition(pub RwLock<String>);

    #[post("/navigator/<id>")]
    pub async fn get_connections(
        id: &str,
        config: &State<Config>,
        cookies: &CookieJar<'_>,
        experience_manager: &State<ExperienceManager>,
        navigator_position: &State<NavigatorPosition>,
    ) -> status::Custom<Json<APIResult<ExperienceConnectionResponse>>> {
        match experience_manager.get_experience(id).await {
            Ok(v) => {
                if let Err(e) = auth(cookies, config) {
                    if v.public {
                        let mut connections = Vec::new();
                        if let Some(connected_experiences) =
                            v.events.get(&AvailablePlugins::timeline_plugin_experience)
                        {
                            {
                                for experience in connected_experiences {
                                    if experience.id != id
                                        && let Ok(exp) =
                                            experience_manager.get_experience(&experience.id).await
                                        && exp.public
                                    {
                                        connections.push(ExperienceConnection {
                                            id: experience.id.clone(),
                                            name: exp.name,
                                        });
                                    }
                                }
                            }
                        }
                        return status::Custom(
                            Status::Ok,
                            Json(Ok(ExperienceConnectionResponse {
                                public: true,
                                connections: Vec::new(),
                                experience_name: v.name,
                            })),
                        );
                    } else {
                        return status::Custom(Status::Unauthorized, Json(Err(e)));
                    };
                }

                *navigator_position.0.write().await = id.to_string();
                let res = v
                    .events
                    .get(&AvailablePlugins::timeline_plugin_experience)
                    .map(|v| {
                        v.iter()
                            .filter_map(|v| {
                                if v.id != id {
                                    Some(ExperienceConnection {
                                        name: v.event.title.clone(),
                                        id: v.id.clone(),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or(Vec::new());
                status::Custom(
                    Status::Ok,
                    Json(Ok(ExperienceConnectionResponse {
                        connections: res,
                        experience_name: v.name.clone(),
                        public: v.public,
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

#[post("/timeline_url")]
pub fn timeline_url(config: &State<Config>) -> status::Accepted<Json<APIResult<String>>> {
    status::Accepted(Json(Ok(config.timeline_url.to_string())))
}

#[post("/auth")]
pub fn auth_request(
    config: &State<Config>,
    cookies: &CookieJar<'_>,
) -> status::Custom<Json<APIResult<()>>> {
    status::Custom(Status::Ok, Json(auth(cookies, config)))
}
