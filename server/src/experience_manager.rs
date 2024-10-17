use {
    chrono::Utc,
    raqote::{DrawOptions, DrawTarget, Image},
    shared::{
        timeline::types::{
            api::{AvailablePlugins, CompressedEvent},
            timing::Timing,
        },
        types::{
            CompressedExperienceEvent, Experience, ExperienceError, ExperienceEvent,
            ExperienceResult,
        },
    },
    std::{collections::HashMap, path::PathBuf, sync::Arc, thread},
    tokio::{
        fs::{write, File},
        io::AsyncReadExt,
        sync::RwLock,
    },
};

use crate::{config::Config, renderer::Renderer};

pub struct ExperienceManager {
    experiences_folder: PathBuf,
    covers_folder: PathBuf,
    cache: RwLock<HashMap<String, Arc<RwLock<Experience>>>>,
    pub renderer: Arc<Renderer>,
}

impl ExperienceManager {
    pub async fn new(config: &Config) -> Self {
        ExperienceManager {
            experiences_folder: config.experiences_folder.clone(),
            cache: RwLock::new(HashMap::new()),
            renderer: Arc::new(Renderer::new().await),
            covers_folder: config.covers_folder.clone(),
        }
    }

    pub async fn get_experience(&self, id: &str) -> ExperienceResult<Experience> {
        let found_experience;
        {
            let cache = self.cache.read().await;
            found_experience = cache.get(id).cloned();
        }

        match found_experience {
            Some(v) => Ok(v.read().await.clone()),
            None => {
                let path = self
                    .experiences_folder
                    .join(format!("{}.experience.json", id));
                let mut file = match File::open(path).await {
                    Ok(v) => v,
                    Err(e) => return Err(ExperienceError::NotFound(e.to_string())),
                };
                let mut experience_file_content = String::new();
                if let Err(e) = file.read_to_string(&mut experience_file_content).await {
                    return Err(ExperienceError::FileError(e.to_string()));
                }

                let experience: Experience = serde_json::from_str(&experience_file_content)?;

                {
                    self.cache
                        .write()
                        .await
                        .insert(id.to_string(), Arc::new(RwLock::new(experience.clone())));
                }

                Ok(experience)
            }
        }
    }

    pub async fn create_experience(&self, name: String, time: Timing) -> ExperienceResult<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let mut events = HashMap::new();
        events.insert(
            AvailablePlugins::timeline_plugin_experience,
            vec![ExperienceEvent {
                event: CompressedEvent {
                    time,
                    title: name.clone(),
                    data: serde_json::to_string(&CompressedExperienceEvent::Experience(id.clone()))
                        .unwrap(),
                },
                favorite: false,
                id: id.clone(),
            }],
        );

        let experience = Experience {
            name,
            events,
            public: false,
        };

        self.save_experience(&id, experience).await?;

        Ok(id)
    }

    pub async fn delete_event(
        &self,
        experience_id: &str,
        event_id: &str,
    ) -> ExperienceResult<Option<(AvailablePlugins, ExperienceEvent)>> {
        if event_id == experience_id {
            return Err(ExperienceError::OperationNowAllowed(
                "Not allowed to delete experience from self".to_string(),
            ));
        }
        let res = self.delete_event_unchecked(experience_id, event_id).await?;
        if let Some(res) = &res
            && res.0 == AvailablePlugins::timeline_plugin_experience
        {
            let connected_to_experience: CompressedExperienceEvent =
                serde_json::from_str(&res.1.event.data)?;
            if let CompressedExperienceEvent::Experience(id) = connected_to_experience {
                self.delete_event_unchecked(&id, experience_id)
                    .await.unwrap_or_else(|e| panic!("Unable to delete a connection: Deleting the connection from the counterpart failed: {}. This is my experience id: {}. This is the experience id of the counter part: {}", e, experience_id, id));
            }
        }
        Ok(res)
    }

    async fn delete_event_unchecked(
        &self,
        experience_id: &str,
        event_id: &str,
    ) -> ExperienceResult<Option<(AvailablePlugins, ExperienceEvent)>> {
        let mut experience = self.get_experience(experience_id).await?;
        let mut deleted_event = None;
        let mut deleted_event_plugin = None;
        experience.events.iter_mut().for_each(|v| {
            v.1.retain(|event| {
                if event.id == event_id {
                    deleted_event = Some((v.0.clone(), event.clone()));
                    deleted_event_plugin = Some(v.0.clone());
                    false
                } else {
                    true
                }
            })
        });
        if let Some(deleted_event_plugin) = deleted_event_plugin
            && let Some(events) = experience.events.get(&deleted_event_plugin)
            && events.is_empty()
        {
            experience.events.remove(&deleted_event_plugin);
        }

        self.save_experience(experience_id, experience).await?;
        Ok(deleted_event)
    }

    pub async fn favorite_event(
        &self,
        experience_id: &str,
        event_id: &str,
        favorite: bool,
    ) -> ExperienceResult<()> {
        let mut experience = self.get_experience(experience_id).await?;
        experience.events.iter_mut().for_each(|v| {
            v.1.iter_mut().for_each(|v| {
                if v.id == event_id {
                    v.favorite = favorite
                }
            })
        });
        self.save_experience(experience_id, experience).await?;
        Ok(())
    }

    pub async fn append_event(
        &self,
        experience_id: &str,
        event: (AvailablePlugins, CompressedEvent),
    ) -> ExperienceResult<String> {
        if event.0 == AvailablePlugins::timeline_plugin_experience {
            let experience_a_id: String =
                match serde_json::from_str::<CompressedExperienceEvent>(&event.1.data)? {
                    CompressedExperienceEvent::Experience(v) => v,
                    CompressedExperienceEvent::Create(_v) => {
                        return self.append_event_unchecked(experience_id, event).await
                    }
                };
            let experience_b_id: String = experience_id.to_string();
            let mut experience_a = self.get_experience(&experience_a_id).await?;
            let mut experience_b = self.get_experience(&experience_b_id).await?;

            let experience_b_time = experience_b
                .events
                .get(&AvailablePlugins::timeline_plugin_experience)
                .and_then(|v| {
                    v.iter()
                        .find(|v| v.id == experience_b_id)
                        .map(|v| v.event.time.clone())
                })
                .unwrap_or(Timing::Instant(Utc::now()));

            let experience_a_experience_event = ExperienceEvent {
                favorite: false,
                id: experience_b_id.clone(),
                event: CompressedEvent {
                    data: serde_json::to_string(&CompressedExperienceEvent::Experience(
                        experience_b_id.clone(),
                    ))?,
                    time: experience_b_time,
                    title: experience_b.name.clone(),
                },
            };
            match experience_a
                .events
                .get_mut(&AvailablePlugins::timeline_plugin_experience)
            {
                None => {
                    experience_a.events.insert(
                        AvailablePlugins::timeline_plugin_experience,
                        vec![experience_a_experience_event],
                    );
                }
                Some(v) => {
                    v.retain(|v| v.id != experience_b_id);
                    v.push(experience_a_experience_event);
                }
            }

            let experience_b_experience_event = ExperienceEvent {
                favorite: false,
                id: experience_a_id.clone(),
                event: CompressedEvent {
                    data: serde_json::to_string(&CompressedExperienceEvent::Experience(
                        experience_a_id.clone(),
                    ))?,
                    time: event.1.time,
                    title: experience_a.name.clone(),
                },
            };
            match experience_b
                .events
                .get_mut(&AvailablePlugins::timeline_plugin_experience)
            {
                None => {
                    experience_b.events.insert(
                        AvailablePlugins::timeline_plugin_experience,
                        vec![experience_b_experience_event],
                    );
                }
                Some(v) => {
                    v.retain(|v| v.id != experience_a_id);
                    v.push(experience_b_experience_event);
                }
            }

            self.save_experience(&experience_a_id, experience_a)
                .await
                .unwrap_or_else(|e| {
                    panic!(
                        "Unable to save experience when creating a connection between them: {}",
                        e
                    )
                });
            self.save_experience(&experience_b_id, experience_b)
                .await
                .unwrap_or_else(|e| {
                    panic!(
                        "Unable to save experience when creating a connection between them: {}",
                        e
                    )
                });

            Ok(experience_a_id)
        } else {
            self.append_event_unchecked(experience_id, event).await
        }
    }

    async fn append_event_unchecked(
        &self,
        experience_id: &str,
        event: (AvailablePlugins, CompressedEvent),
    ) -> ExperienceResult<String> {
        let mut experience = self.get_experience(experience_id).await?;

        let id = uuid::Uuid::new_v4().to_string();
        let experience_event = ExperienceEvent {
            id: id.clone(),
            favorite: false,
            event: event.1,
        };

        match experience.events.get_mut(&event.0) {
            Some(v) => v.push(experience_event),
            None => {
                experience.events.insert(event.0, vec![experience_event]);
            }
        }

        self.save_experience(experience_id, experience).await?;

        Ok(id)
    }

    pub async fn set_experience_visibility(
        &self,
        id: &str,
        visibility: bool,
    ) -> ExperienceResult<()> {
        let mut experience = self.get_experience(id).await?;
        experience.public = visibility;

        self.save_experience(id, experience).await
    }

    async fn save_experience(&self, id: &str, experience: Experience) -> ExperienceResult<()> {
        self.write_experience(id, &experience).await?;

        {
            self.cache
                .write()
                .await
                .insert(id.to_string(), Arc::new(RwLock::new(experience.clone())));
        }

        Ok(())
    }

    async fn write_experience(&self, id: &str, experience: &Experience) -> ExperienceResult<()> {
        let path = self
            .experiences_folder
            .join(format!("{}.experience.json", id));
        let res = match write(path, serde_json::to_string(experience)?).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ExperienceError::UnableToWrite(e.to_string())),
        };
        self.generate_experience_cover(id.to_string(), experience.clone());
        res
    }

    fn generate_experience_cover(&self, id: String, experience: Experience) {
        let renderer = self.renderer.clone();
        let covers_folder = self.covers_folder.clone();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        thread::spawn(move || {
            let local = tokio::task::LocalSet::new();

            local.block_on(&rt, async move {
                let _ = tokio::task::spawn_local(async move {
                    let dt = renderer.render_experience(&experience, 500).await;
                    if let Err(e) = dt.write_png(covers_folder.join(format!("{}.png", id))) {
                        eprintln!("Unable to save big cover: {}", e);
                    }

                    let mut small_dt = DrawTarget::new(100, 100);
                    let pixel_data = dt.get_data();
                    let image = Image {
                        width: 500,
                        height: 500,
                        data: pixel_data,
                    };
                    small_dt.draw_image_with_size_at(
                        100.,
                        100.,
                        0.,
                        0.,
                        &image,
                        &DrawOptions::new(),
                    );
                    if let Err(e) =
                        small_dt.write_png(covers_folder.join(format!("{}.small.png", id)))
                    {
                        eprintln!("Unable to save small cover: {}", e);
                    }
                })
                .await;
            });
        });
    }
}
