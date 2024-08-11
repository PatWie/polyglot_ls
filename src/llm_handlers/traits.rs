use super::bedrock::BedrockConverse;
use super::mock::MockLLM;

pub trait LlmHandler {
    fn answer(
        &self,
        prompt: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<String>> + Send;
}

pub enum Llm {
    Bedrock(BedrockConverse),
    Mock(MockLLM),
}

impl Llm {
    pub async fn answer<'a>(&'a self, prompt: &'a str) -> anyhow::Result<String> {
        match self {
            Llm::Bedrock(b) => b.answer(prompt).await,
            Llm::Mock(b) => b.answer(prompt).await,
        }
    }
}
