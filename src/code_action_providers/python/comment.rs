use async_trait::async_trait;
use serde_json::json;
use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{CodeAction, CodeActionKind, TextEdit, WorkspaceEdit};

use crate::code_action_providers::helper::{self, findup};
use crate::code_action_providers::parsed_document::ParsedDocument;
use crate::code_action_providers::traits::ActionContext;
use crate::code_action_providers::traits::ActionProvider;
use crate::prompt_handlers::claude::BedrockConverse;
use crate::prompt_handlers::traits::PromptHandler;
use crate::ResolveAction;

pub const ENHANCE_FUNCTION_DOCSTRING_ACTION: &str = "povider.py.comment";

fn build_prompt(function_text: &str) -> String {
    let pre_prompt: String = r#"
Human: Improve the comment by better grammer, fixing typos and concise expression.
    ONLY output the comment without explanations. Do not wrap it in any markdown. Just return the comment. Keep the start "\#" as it is a comment.
    <task>"#.to_owned();
    let post_prompt: String = r#"
</task>
    Assistant:

"#
    .to_owned();
    format!("{}\n{}\n{}", pre_prompt, function_text, post_prompt)
}

pub struct EnhanceCommentProvider {
    prompt_handler: BedrockConverse,
}

impl EnhanceCommentProvider {
    pub async fn new() -> Self {
        Self {
            prompt_handler: BedrockConverse::new().await.unwrap(),
        }
    }
}
#[async_trait]
impl ActionProvider for EnhanceCommentProvider {
    fn can_handle(&self, action_name: &str) -> bool {
        action_name == ENHANCE_FUNCTION_DOCSTRING_ACTION
    }
    async fn on_resolve(&self, doc: &ParsedDocument, action: CodeAction) -> Result<CodeAction> {
        let args =
            serde_json::from_value::<ResolveAction<ActionContext>>(action.data.clone().unwrap())
                .unwrap()
                .data;

        let comment_node = doc.get_ts_node_for_range(&args.ctx_range);

        if let Some(comment_node) = comment_node {
            let comment_text = doc.get_text(&comment_node);
            let mut generated_text = self
                .prompt_handler
                .answer(&build_prompt(&comment_text))
                .await
                .unwrap();

            let target_range = helper::ts_node_to_lsp_range(&comment_node);

            let text_edit = TextEdit {
                range: target_range,
                new_text: generated_text,
            };
            let mut action = action.clone();
            action.edit = Some(WorkspaceEdit {
                changes: Some([(args.uri.clone(), vec![text_edit])].into_iter().collect()),
                ..Default::default()
            });

            Ok(action)
        } else {
            Err(Error::new(tower_lsp::jsonrpc::ErrorCode::ParseError))
        }
    }
    fn create_code_action(
        &self,
        doc: &ParsedDocument,
        start_range: &tower_lsp::lsp_types::Range,
    ) -> Option<tower_lsp::lsp_types::CodeAction> {
        let cursor_node = doc.get_ts_node_for_range(start_range);
        let ctx_node = findup(cursor_node, "comment");

        if let Some(ctx_node) = ctx_node {
            let function_range = helper::ts_node_to_lsp_range(&ctx_node);

            return Some(CodeAction {
                title: "Update Comment".to_string(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE),
                data: Some(json!(ResolveAction {
                    kind: ENHANCE_FUNCTION_DOCSTRING_ACTION.to_string(),
                    data: ActionContext {
                        uri: doc.uri.to_owned(),
                        ctx_range: function_range
                    }
                })),
                ..Default::default()
            });
        }
        None
    }
}
