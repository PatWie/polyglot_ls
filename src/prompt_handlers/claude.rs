use aws_config::{BehaviorVersion, Region};
use aws_sdk_bedrockruntime::{
    operation::converse::{ConverseError, ConverseOutput},
    types::{ContentBlock, ConversationRole, Message},
    Client,
};

use crate::configuration::BedrockConfig;

use super::traits::PromptHandler;

// Set the model ID, e.g., Claude 3 Haiku.
// const MODEL_ID: &str = "anthropic.claude-3-haiku-20240307-v1:0";
// const CLAUDE_REGION: &str = "us-east-1";
// const AWS_PROFILE: &str = "my-aws-bedrock";
// Start a conversation with the user message.

#[derive(Debug)]
pub struct BedrockConverse {
    client: Client,
    model_id: String,
}

impl BedrockConverse {
    pub async fn new(config: &BedrockConfig) -> Result<Self, BedrockConverseError> {
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .profile_name(config.aws_profile.clone())
            .load()
            .await;

        let client = Client::new(&sdk_config);

        Ok(BedrockConverse {
            client,
            model_id: config.model_id.clone(),
        })
    }
}

impl PromptHandler for BedrockConverse {
    type Error = BedrockConverseError;

    async fn answer(&self, prompt: &str) -> Result<String, Self::Error> {
        let response = self
            .client
            .converse()
            .model_id(&self.model_id)
            .messages(
                Message::builder()
                    // .role(ConversationRole::Assistant)
                    // .content(ContentBlock::Text(prompt.to_string()))
                    .role(ConversationRole::User)
                    .content(ContentBlock::Text(prompt.to_string()))
                    .build()
                    .map_err(|_| "failed to build message")?,
            )
            .send()
            .await;

        match response {
            Ok(output) => get_converse_output_text(output),
            Err(e) => Err(e
                .as_service_error()
                .map(BedrockConverseError::from)
                .unwrap_or_else(|| BedrockConverseError("Unknown service error".into()))),
        }
    }
}

#[derive(Debug)]
pub struct BedrockConverseError(String);

impl std::fmt::Display for BedrockConverseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Can't invoke model. Reason: {}", self.0)
    }
}

impl std::error::Error for BedrockConverseError {}

impl From<&str> for BedrockConverseError {
    fn from(value: &str) -> Self {
        BedrockConverseError(value.to_string())
    }
}

impl From<&ConverseError> for BedrockConverseError {
    fn from(value: &ConverseError) -> Self {
        BedrockConverseError::from(match value {
            ConverseError::ModelTimeoutException(_) => "Model took too long",
            ConverseError::ModelNotReadyException(_) => "Model is not ready",
            _ => "Unknown",
        })
    }
}

fn get_converse_output_text(output: ConverseOutput) -> Result<String, BedrockConverseError> {
    let text = output
        .output()
        .ok_or("no output")?
        .as_message()
        .map_err(|_| "output not a message")?
        .content()
        .first()
        .ok_or("no content in message")?
        .as_text()
        .map_err(|_| "content is not text")?
        .to_string();
    Ok(text)
}
