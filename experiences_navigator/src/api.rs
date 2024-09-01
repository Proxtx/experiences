use {
    experiences_types_lib::types::ExperiencesHostname,
    leptos::use_context,
    reqwest::Client,
    types::api::APIResult,
    url::{ParseError, Url},
};

pub async fn api_request<T, V>(endpoint: &str, request: &V) -> APIResult<T>
where
    T: serde::de::DeserializeOwned,
    V: serde::Serialize,
{
    let client = Client::new();
    let url = relative_url(&format!("/api{}", endpoint)).unwrap();
    serde_json::from_str::<APIResult<T>>(
        &client
            .post(url)
            .body(serde_json::to_string(request)?)
            .fetch_credentials_include()
            .send()
            .await?
            .text()
            .await?,
    )?
}

pub fn relative_url(path: &str) -> Result<Url, ParseError> {
    let experiences_host: ExperiencesHostname = use_context().unwrap();
    Url::parse(&experiences_host.0)?.join(path)
}
