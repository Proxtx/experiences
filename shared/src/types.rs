use std::{collections::HashMap, error, fmt};

use serde::{Deserialize, Serialize};

use crate::timeline::types::api::{AvailablePlugins, CompressedEvent};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExperienceEvent {
    pub favorite: bool,
    pub id: String,
    pub event: CompressedEvent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Experience {
    pub events: HashMap<AvailablePlugins, Vec<ExperienceEvent>>,
    pub public: bool,
    pub name: String,
}

#[cfg_attr(feature = "server", derive(Serialize))]
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Debug)]
pub enum ExperienceError {
    NotFound(String),
    FileError(String),
    ParsingError(String),
    UnableToWrite(String),
}

impl fmt::Display for ExperienceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExperienceError::NotFound(v) => write!(f, "Could not find experience: {}", v),
            ExperienceError::UnableToWrite(v) => {
                write!(f, "Unable to write experience to file: {}", v)
            }
            ExperienceError::ParsingError(v) => write!(f, "Unable to parse data: {}", v),
            ExperienceError::FileError(v) => write!(f, "Unable to read the file correctly: {}", v),
        }
    }
}

pub type ExperienceResult<T> = Result<T, ExperienceError>;

#[cfg_attr(feature = "server", derive(Serialize))]
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Debug, Clone)]
pub struct ExperienceConnection {
    pub id: String,
    pub name: String,
}

#[cfg_attr(feature = "server", derive(Serialize))]
#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Debug, Clone)]
pub struct ExperienceConnectionResponse {
    pub connections: Vec<ExperienceConnection>,
    pub experience_name: String,
}

impl error::Error for ExperienceError {}

#[cfg(feature = "server")]
impl From<serde_json::Error> for ExperienceError {
    fn from(value: serde_json::Error) -> Self {
        ExperienceError::ParsingError(value.to_string())
    }
}
