use {
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
            .send()
            .await?
            .text()
            .await?,
    )?
}

pub fn relative_url(path: &str) -> Result<Url, ParseError> {
    Url::parse(&leptos::window().origin())?.join(path)
}
