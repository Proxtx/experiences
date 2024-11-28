use {
    serde::{Deserialize, Serialize},
    types::timing::Timing,
};

#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Debug, Clone, Serialize)]
pub struct ExperienceConnection {
    pub id: String,
    pub name: String,
}

#[cfg_attr(feature = "client", derive(Deserialize))]
#[derive(Debug, Clone, Serialize)]
pub struct ExperienceConnectionResponse {
    pub connections: Vec<ExperienceConnection>,
    pub experience_name: String,
    pub public: bool,
}

pub use types::api::ExperiencesHostname;

#[derive(Serialize, Deserialize)]
pub enum CompressedExperienceEvent {
    Experience(String),
    Create(Timing),
}

#[derive(Serialize, Deserialize)]
pub struct CreateExperienceRequest {
    pub name: String,
    pub time: Timing,
}
