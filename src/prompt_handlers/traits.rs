pub trait PromptHandler {
    type Error: std::error::Error;

    fn answer(
        &self,
        prompt: &str,
    ) -> impl std::future::Future<Output = Result<String, Self::Error>> + Send;
}
