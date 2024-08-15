use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use shared::types::{Experience, ExperienceError, ExperienceResult};
use tokio::sync::RwLock;

use crate::config::Config;

pub struct ExperienceManager {
    folder: PathBuf,
    cache: RwLock<HashMap<String, RwLock<Experience>>>,
}

impl ExperienceManager {
    pub fn new(config: &Config) -> Self {
        ExperienceManager {
            folder: config.experiences_folder.clone(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_experience(&self, id: &str) -> ExperienceResult<Experience> {
        {
            let cache = self.cache.read().await;
            if let Some(v) = cache.get(id) {
                return Ok(v.read().await.clone());
            }
        }

        unimplemented!()
    }
}
