use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{CodeAction, CodeActionKind, TextEdit, WorkspaceEdit};

use crate::code_action_providers::traits::ActionContext;
use crate::code_action_providers::traits::ActionProvider;
use crate::code_action_providers::{helper, parsed_document::ParsedDocument};
use crate::prompt_handlers::claude::BedrockConverse;
use crate::prompt_handlers::traits::PromptHandler;
use crate::ResolveAction;

use super::config;

fn build_prompt(template: &str, hints: &HashMap<String, String>) -> String {
    let mut prompt = template.to_owned();
    for (name, hint) in hints {
        prompt = prompt.replace(&format!("<<<{}>>>", name), hint);
    }
    prompt.to_owned()
}

pub struct GenericProvider {
    prompt_handler: Arc<BedrockConverse>,

    config: config::CodeAction,
    id: String,
}

impl GenericProvider {
    pub fn from_config(
        config: config::CodeAction,
        id: &str,
        prompt_handler: Arc<BedrockConverse>,
    ) -> Self {
        Self {
            prompt_handler,
            config,
            id: id.to_owned(),
        }
    }
}
#[async_trait]
impl ActionProvider for GenericProvider {
    fn can_handle(&self, action_name: &str) -> bool {
        action_name == self.id
    }
    async fn on_resolve(&self, doc: &ParsedDocument, action: CodeAction) -> Result<CodeAction> {
        let args =
            serde_json::from_value::<ResolveAction<ActionContext>>(action.data.clone().unwrap())
                .unwrap()
                .data;

        let ctx_node = doc.get_ts_node_for_range(&args.ctx_range);

        if let Some(ctx_node) = ctx_node {
            let mut hint_texts: HashMap<String, String> = Default::default();

            for hint in self.config.context.hints.iter() {
                let hint_node = doc.find_first(&ctx_node, &hint.query);
                if let Some(hint_node) = hint_node {
                    let hint_text = doc.get_text(&hint_node);
                    hint_texts.insert(hint.name.clone(), hint_text);
                }
                // let function_text = doc.get_text(&function_node);
            }
            log::info!("hints {:?}", hint_texts);

            let prompt = build_prompt(&self.config.prompt_template, &hint_texts);
            log::info!("prompt {}", prompt);
            let mut answer = self.prompt_handler.answer(&prompt).await.unwrap();
            if let Some(answer_template) = self.config.answer_template.clone() {
                answer = answer_template.replace("<<<ANSWER>>>", &answer);
            }
            log::info!("answer {}", answer);

            for placement in self.config.placement_strategies.iter() {
                let placement_node = doc.find_first(&ctx_node, &placement.query);
                if let Some(placement_node) = placement_node {
                    log::info!("placement {:?}", placement);
                    let (range, new_text) = match placement.position {
                        config::Position::ReplaceBlock => {
                            let mut target_range = helper::ts_node_to_lsp_range(&placement_node);
                            target_range.start.character = 0;
                            let new_text = helper::indent_text(
                                &answer,
                                placement_node.range().start_point.column,
                            );
                            (target_range, new_text)
                        }
                        config::Position::ReplaceExact => {
                            let target_range = helper::ts_node_to_lsp_range(&placement_node);
                            (target_range, answer)
                        }
                        config::Position::Before => {
                            let target_range =
                                helper::prepend_ts_node_to_lsp_range(&placement_node);
                            let new_text = format!(
                                "{}\n",
                                helper::indent_text(
                                    &answer,
                                    placement_node.range().start_point.column,
                                )
                            );
                            (target_range, new_text)
                        }
                    };

                    log::info!("new_text {:?}", new_text);
                    let text_edit = TextEdit { range, new_text };
                    let mut action = action.clone();
                    action.edit = Some(WorkspaceEdit {
                        changes: Some([(args.uri.clone(), vec![text_edit])].into_iter().collect()),
                        ..Default::default()
                    });

                    return Ok(action);
                }
            }
        }
        return Err(Error::new(tower_lsp::jsonrpc::ErrorCode::ParseError));
    }
    fn create_code_action(
        &self,
        doc: &ParsedDocument,
        start_range: &tower_lsp::lsp_types::Range,
    ) -> Option<tower_lsp::lsp_types::CodeAction> {
        let cursor_node = doc.get_ts_node_for_range(start_range);
        log::info!("Cursor node {:?}", cursor_node);

        let is_triggered = self
            .config
            .triggers
            .iter()
            .any(|trigger| trigger.is_triggered(cursor_node));
        log::info!("is_triggered {:?}", is_triggered);

        if !is_triggered {
            return None;
        }

        let context_node = self.config.context.find(cursor_node);
        if let Some(context_node) = context_node {
            let ctx_range = helper::ts_node_to_lsp_range(&context_node);
            return Some(CodeAction {
                title: format!("Polyglot: {}", self.config.name),
                kind: Some(CodeActionKind::REFACTOR_REWRITE),
                data: Some(json!(ResolveAction {
                    id: self.id.to_string(),
                    data: ActionContext {
                        uri: doc.uri.to_owned(),
                        ctx_range
                    }
                })),
                ..Default::default()
            });
        }
        None
    }
}
