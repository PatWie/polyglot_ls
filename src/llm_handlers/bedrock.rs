use anyhow::bail;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_bedrockruntime::{
    operation::converse::{ ConverseOutput},
    types::{ContentBlock, ConversationRole, Message},
    Client,
};

use crate::configuration::BedrockConfig;

use super::traits::LlmHandler;


#[derive(Debug)]
pub struct BedrockConverse {
    client: Client,
    model_id: String,
}

impl BedrockConverse {
    pub async fn new(config: &BedrockConfig) -> anyhow::Result<Self> {
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

impl LlmHandler for BedrockConverse {
    async fn answer(&self, prompt: &str) -> anyhow::Result<String> {
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
                    .build()?, // .map_err(|_| "failed to build message")?,
            )
            .send()
            .await;
        let e = get_converse_output_text(response?);
        match e {
            Ok(s) => Ok(s),
            Err(_) => bail!("failed to get response"),
        }
    }
}
fn get_converse_output_text(output: ConverseOutput) -> Result<String, String> {
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
