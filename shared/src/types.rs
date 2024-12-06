use std::{collections::HashMap, error, fmt, hash::Hash};

use serde::{Deserialize, Serialize};

use timeline_types::{
    api::{CompressedEvent, EventWrapper},
    available_plugins::AvailablePlugins,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ExperienceEvent {
    pub favorite: bool,
    pub id: String,
    pub event: CompressedEvent,
}

impl EventWrapper for ExperienceEvent {
    fn get_compressed_event(&self) -> CompressedEvent {
        self.event.clone()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct PluginExperienceEvent(pub AvailablePlugins, pub ExperienceEvent);

impl EventWrapper for PluginExperienceEvent {
    fn get_compressed_event(&self) -> CompressedEvent {
        self.1.get_compressed_event()
    }
}

#[derive(Serialize, Deserialize)]
pub struct FavoriteRequest {
    pub event_id: String,
    pub favorite: bool,
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
    OperationNowAllowed(String),
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
            ExperienceError::OperationNowAllowed(v) => {
                write!(f, "The performed operation is not allowed: {}", v)
            }
        }
    }
}

pub type ExperienceResult<T> = Result<T, ExperienceError>;

impl error::Error for ExperienceError {}

#[cfg(feature = "server")]
impl From<serde_json::Error> for ExperienceError {
    fn from(value: serde_json::Error) -> Self {
        ExperienceError::ParsingError(value.to_string())
    }
}

pub use experiences_types_lib::types::CreateExperienceRequest;

pub use experiences_types_lib::types::CompressedExperienceEvent;
