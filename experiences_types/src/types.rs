use serde::{Deserialize, Serialize};

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
}
