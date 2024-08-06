use std::fs;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use thiserror::Error;
use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::{CodeAction, CodeActionKind, TextEdit, WorkspaceEdit};

use crate::code_action_providers::parsed_document::ParsedDocument;
use crate::code_action_providers::traits::ActionContext;
use crate::code_action_providers::traits::ActionProvider;
use crate::prompt_handlers::claude::BedrockConverse;
use crate::prompt_handlers::traits::PromptHandler;
use crate::ResolveAction;

use super::lua_jit::LuaInterface;

pub struct LuaProvider {
    prompt_handler: Arc<BedrockConverse>,
    lua_source: String,
    id: String,
}

#[derive(Debug, Error)]
pub enum LuaProviderError {
    #[error("Error reading Lua file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Error parsing Lua source: {0}")]
    LuaParseError(String),
}

impl LuaProvider {
    pub fn try_new(
        file_name: &str,
        prompt_handler: Arc<BedrockConverse>,
    ) -> anyhow::Result<Self, LuaProviderError> {
        Ok(Self {
            prompt_handler,
            id: file_name.to_owned(),
            lua_source: fs::read_to_string(file_name)?,
        })
    }

    pub fn create_lua(&self, doc: &ParsedDocument) -> LuaInterface {
        let lua = LuaInterface::new(&self.lua_source);
        lua.set_doc(doc.duplicate());
        lua
    }
}
#[async_trait]
impl ActionProvider for LuaProvider {
    fn can_handle(&self, action_name: &str) -> bool {
        action_name == self.id
    }
    async fn on_resolve(&self, doc: &ParsedDocument, action: CodeAction) -> Result<CodeAction> {
        let args = serde_json::from_value::<ResolveAction<ActionContext>>(
            action.data.clone().expect("action data is correct"),
        )
        .expect("can parse action data")
        .data;

        let prompt;
        let range;
        {
            let lua = self.create_lua(doc);
            range = lua
                .placement_range(&args.ctx_range)
                .ok_or(Error::request_cancelled())?;
            prompt = lua
                .build_prompt(&args.ctx_range)
                .ok_or(Error::request_cancelled())?;
        }
        let mut new_text: String = self.prompt_handler.answer(&prompt).await.unwrap();
        {
            log::info!("answer {}", new_text);
            let lua = self.create_lua(doc);
            new_text = lua
                .process_answer(&new_text, &args.ctx_range)
                .ok_or(Error::request_cancelled())?;
            log::info!("processed answer {}", new_text);
        }
        let text_edit = TextEdit { range, new_text };
        let mut action = action.clone();
        action.edit = Some(WorkspaceEdit {
            changes: Some([(args.uri.clone(), vec![text_edit])].into_iter().collect()),
            ..Default::default()
        });

        return Ok(action);
    }
    fn create_code_action(
        &self,
        doc: &ParsedDocument,
        start_range: &tower_lsp::lsp_types::Range,
    ) -> Option<tower_lsp::lsp_types::CodeAction> {
        let lua = self.create_lua(doc);
        let is_triggered = lua.is_triggered(start_range);
        if !is_triggered {
            return None;
        }
        let lua = self.create_lua(doc);

        Some(CodeAction {
            title: format!("Polyglot: {}", lua.action_name().unwrap()),
            kind: Some(CodeActionKind::REFACTOR_REWRITE),
            data: Some(json!(ResolveAction {
                id: self.id.to_string(),
                data: ActionContext {
                    uri: doc.uri.to_owned(),
                    ctx_range: start_range.to_owned()
                }
            })),
            ..Default::default()
        })
    }
}
