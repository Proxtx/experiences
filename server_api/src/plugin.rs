pub trait PluginRenderer: Send + Sync {
    fn new() -> impl std::future::Future<Output = Self> + Send
    where
        Self: Sized;
    fn render(
        &self,
        dimensions: (i32, i32),
        event: &CompressedEvent,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<Vec<u32>, String>> + Send>>;
}
