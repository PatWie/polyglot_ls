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

pub const ENHANCE_FUNCTION_DOCSTRING_ACTION: &str = "povider.py.func_args";

const PARAMETER_TS_QUERY: &str = r#"
(function_definition
        parameters: (parameters) @parameters)
"#;

fn build_prompt(function_text: &str) -> String {
    let pre_prompt: String = r#"
Human: Enhance the function parameters by updating or adding python3 type annotations

for
    def fetch_smalltable_rows( table_handle, keys,
        require_all_keys: bool = False,
    ):

a version with annotations might look like

    def fetch_smalltable_rows(
        table_handle: smalltable.Table,
        keys: Sequence[bytes | str],
        require_all_keys: bool = False,
    ) -> Mapping[bytes, tuple[str, ...]]:


    Use the correct type by understand the function body. Do NOT use "Any" if you can derive the correct type from the function body.
    If there are pre-existing default values, keep them as they are oif they make sense.
    Remember, class methods start with "self" as first arguments without an annotation. Keep pre-existing "self" args.
    ONLY output the parameters comma-separated, without function name and parenthese

    Here is the task:
    <task>"#.to_owned();
    let post_prompt: String = r#"
</task>
    Assistant:

"#
    .to_owned();
    format!("{}\n{}\n{}", pre_prompt, function_text, post_prompt)
}

pub struct EnhanceFunctionArgsProvider {
    prompt_handler: BedrockConverse,
}

impl EnhanceFunctionArgsProvider {
    pub async fn new() -> Self {
        Self {
            prompt_handler: BedrockConverse::new().await.unwrap(),
        }
    }
}
#[async_trait]
impl ActionProvider for EnhanceFunctionArgsProvider {
    fn can_handle(&self, action_name: &str) -> bool {
        action_name == ENHANCE_FUNCTION_DOCSTRING_ACTION
    }
    async fn on_resolve(&self, doc: &ParsedDocument, action: CodeAction) -> Result<CodeAction> {
        let args =
            serde_json::from_value::<ResolveAction<ActionContext>>(action.data.clone().unwrap())
                .unwrap()
                .data;

        let function_node = doc.get_ts_node_for_range(&args.ctx_range);

        if let Some(function_node) = function_node {
            let function_text = doc.get_text(&function_node);
            let mut generated_text = self
                .prompt_handler
                .answer(&build_prompt(&function_text))
                .await
                .unwrap();

            let parameter_node = doc.find_first(&function_node, PARAMETER_TS_QUERY);

            let target_range = helper::ts_node_to_lsp_range(&parameter_node.unwrap());

            // for line in generated_docstring_text.lines() {
            //     println!("line {}", line);
            // }

            let text_edit = TextEdit {
                range: target_range,
                new_text: format!("({})", generated_text),
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
        let function_node = findup(cursor_node, "function_definition");

        if let Some(function_node) = function_node {
            let function_range = helper::ts_node_to_lsp_range(&function_node);

            return Some(CodeAction {
                title: "Update Function Args".to_string(),
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
