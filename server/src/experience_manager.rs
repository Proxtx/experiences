use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::Utc;
use shared::{
    timeline::types::api::{AvailablePlugins, CompressedEvent},
    types::{Experience, ExperienceError, ExperienceEvent, ExperienceResult},
};
use tokio::{
    fs::{write, File},
    io::AsyncReadExt,
    sync::RwLock,
};

use crate::config::Config;

pub struct ExperienceManager {
    folder: PathBuf,
    cache: RwLock<HashMap<String, Arc<RwLock<Experience>>>>,
}

impl ExperienceManager {
    pub fn new(config: &Config) -> Self {
        ExperienceManager {
            folder: config.experiences_folder.clone(),
            cache: RwLock::new(HashMap::new()),
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
                let path = self.folder.join(format!("{}.experience.json", id));
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

    pub async fn create_experience(&self, name: String) -> ExperienceResult<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let experience = Experience {
            name,
            events: HashMap::new(),
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
        let res = self.delete_event_unchecked(experience_id, event_id).await?;
        if let Some(res) = &res
            && res.0 == AvailablePlugins::timeline_plugin_experience
        {
            let connected_to_experience_id: String = serde_json::from_str(&res.1.event.data)?;
            self.delete_event_unchecked(experience_id, experience_id)
                .await.unwrap_or_else(|e| panic!("Unable to delete a connection: Deleting the connection from the counterpart failed: {}. This is my experience id: {}. This is the experience id of the counter part: {}", e, experience_id, connected_to_experience_id));
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
        experience.events.iter_mut().for_each(|v| {
            v.1.retain(|event| {
                if event.id == event_id {
                    deleted_event = Some((v.0.clone(), event.clone()));
                    false
                } else {
                    true
                }
            })
        });
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
            let experience_a_id: String = serde_json::from_str(&event.1.data)?;
            let experience_b_id: String = experience_id.to_string();
            let mut experience_a = self.get_experience(&experience_a_id).await?;
            let mut experience_b = self.get_experience(&experience_b_id).await?;

            let experience_a_experience_event = ExperienceEvent {
                favorite: false,
                id: experience_b_id.clone(),
                event: CompressedEvent {
                    data: serde_json::to_string(&experience_b_id)?,
                    time: shared::timeline::types::timing::Timing::Instant(Utc::now()),
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
                    data: serde_json::to_string(&experience_a_id)?,
                    time: shared::timeline::types::timing::Timing::Instant(Utc::now()),
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

            Ok(experience_b_id)
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
        let path = self.folder.join(format!("{}.experience.json", id));
        match write(path, serde_json::to_string(experience)?).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ExperienceError::UnableToWrite(e.to_string())),
        }
    }
}
