use crate::PluginRenderers;
use raqote::DrawTarget;
use shared::{
    timeline::types::api::{AvailablePlugins, CompressedEvent},
    types::Experience,
};
use std::pin::Pin;

pub trait PluginRenderer: Send + Sync {
    fn new() -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn render(
        &self,
        target: &mut DrawTarget,
        event: &CompressedEvent,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;
}

pub struct Renderer<'a> {
    plugins: PluginRenderers<'a>,
}

impl<'a> Renderer<'a> {
    pub async fn new() -> Renderer<'a> {
        Renderer {
            plugins: PluginRenderers::init().await,
        }
    }

    pub async fn render_event(
        &self,
        plugin: &AvailablePlugins,
        event: &CompressedEvent,
    ) -> DrawTarget {
        let mut draw_target = DrawTarget::new(100, 100);
        if let Some(renderer) = self.plugins.renderers.get(plugin) {
            if let Err(e) = renderer.render(&mut draw_target, event).await {
                eprintln!("Error: Unable to render experience picture: {}", e);
            }
        }
        draw_target
    }

    pub async fn render_experience(&self, experience: Experience) -> DrawTarget {}
}
