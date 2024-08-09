use super::traits::PromptHandler;

#[derive(Debug)]
pub struct MockLLM {
    answer: String,
}

impl MockLLM {
    pub fn new(answer: String) -> anyhow::Result<Self> {
        Ok(MockLLM { answer })
    }
}

impl PromptHandler for MockLLM {
    async fn answer(&self, _: &str) -> anyhow::Result<String> {
        Ok(self.answer.clone())
    }
}
