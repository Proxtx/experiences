use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

use shared::types::{Experience, ExperienceError, ExperienceResult};
use tokio::{fs::File, io::AsyncReadExt, sync::RwLock};

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
        let mut found_experience;
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
}
