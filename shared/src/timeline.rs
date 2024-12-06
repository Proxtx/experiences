impl From<crate::types::ExperienceError> for timeline_types::api::APIError {
    fn from(value: crate::types::ExperienceError) -> Self {
        Self::ExperienceError(value.to_string())
    }
}
