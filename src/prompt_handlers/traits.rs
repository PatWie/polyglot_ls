use super::bedrock::BedrockConverse;
use super::mock::MockLLM;

pub trait PromptHandler {
    fn answer(
        &self,
        prompt: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<String>> + Send;
}

pub enum LLM {
    Bedrock(BedrockConverse),
    Mock(MockLLM),
}

impl LLM {
    pub async fn answer<'a>(&'a self, prompt: &'a str) -> anyhow::Result<String> {
        match self {
            LLM::Bedrock(b) => b.answer(prompt).await,
            LLM::Mock(b) => b.answer(prompt).await,
        }
    }
}
