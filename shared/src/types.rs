use std::{collections::HashMap, error, fmt};

use crate::timeline::types::api::{AvailablePlugins, CompressedEvent};

#[derive(Debug, Clone)]
pub struct ExperienceEvent {
    pub favorite: bool,
    pub event: CompressedEvent,
}

#[derive(Debug, Clone)]
pub struct Experience {
    pub events: HashMap<AvailablePlugins, Vec<ExperienceEvent>>,
    pub public: bool,
}

#[derive(Debug)]
pub enum ExperienceError {
    NotFound(String),
    UnableToWrite(String),
}

impl fmt::Display for ExperienceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExperienceError::NotFound(v) => write!(f, "Could not find experience: {}", v),
            ExperienceError::UnableToWrite(v) => {
                write!(f, "Unable to write experience to file: {}", v)
            }
        }
    }
}

pub type ExperienceResult<T> = Result<T, ExperienceError>;

impl error::Error for ExperienceError {}
