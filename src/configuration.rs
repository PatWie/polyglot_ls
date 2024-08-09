use serde::{Deserialize, Serialize};

use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct PolyglotConfig {
    pub model: ModelConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModelConfig {
    pub bedrock: BedrockConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BedrockConfig {
    pub model_id: String,
    pub region: String,
    pub aws_profile: String,
}

impl Default for PolyglotConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig {
                bedrock: BedrockConfig {
                    model_id: "anthropic.claude-3-haiku-20240307-v1:0".to_string(),
                    region: "us-east-1".to_string(),
                    aws_profile: "my-aws-bedrock".to_string(),
                },
            },
        }
    }
}

impl PolyglotConfig {
    pub fn default_file_path() -> PathBuf {
        let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_path = PathBuf::from(home_dir).join("./config/polyglot_ls.yaml");
        config_path
    }

    pub fn try_read_from_file<P: AsRef<Path>>(config_path: P) -> anyhow::Result<Self> {
        if config_path.as_ref().exists() {
            match fs::read_to_string(&config_path) {
                Ok(config_data) => {
                    let cfg: PolyglotConfig = serde_yaml::from_str(&config_data)?;
                    cfg
                }
                Err(err) => anyhow::bail!(err),
            };
        }
        anyhow::bail!("path does not exists")
    }
}
