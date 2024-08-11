use super::traits::LlmHandler;

#[derive(Debug)]
pub struct MockLLM {
    pub answer: String,
}

impl MockLLM {
    pub fn new(answer: String) -> anyhow::Result<Self> {
        Ok(MockLLM { answer })
    }
}

impl LlmHandler for MockLLM {
    async fn answer(&self, _: &str) -> anyhow::Result<String> {
        Ok(self.answer.clone())
    }
}
