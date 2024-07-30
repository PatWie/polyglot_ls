pub trait PromptHandler {
    type Error: std::error::Error;

    async fn answer(&self, prompt: &str) -> Result<String, Self::Error>;
}
