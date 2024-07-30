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

pub const ENHANCE_FUNCTION_DOCSTRING_ACTION: &str = "povider.py.class_docstring";

const DOCSTRING_TS_QUERY: &str = r#"
(class_definition
    body: (block
      (expression_statement
        (string) @docstring)))"#;
const BODY_TS_QUERY: &str = r#"
(class_definition
    body: (block) @body)
"#;

fn build_prompt(function_text: &str) -> String {
    let pre_prompt: String = r#"
Human: Write a google style docstring for a given class not a function. JUST the class. Here is an example
    for

    class ExampleClass(object):


    this is how it can look like

        """The summary line for a class docstring should fit on one line.

        If the class has public attributes, they may be documented here
        in an ``Attributes`` section and follow the same formatting as a
        function's ``Args`` section. Alternatively, attributes may be documented
        inline with the attribute's declaration (see __init__ method below).

        Properties created with the ``@property`` decorator should be documented
        in the property's getter method.

        Attributes:
            attr1 (str): Description of `attr1`.
            attr2 (:obj:`int`, optional): Description of `attr2`.

        """

    NEVER write anything else besides the docstring block. No markdown like "```python". ONLY generate the docstring.
    It should include a summary of what th class is doing and attributes if necessary. First line must be in imperative mood. Do NOT output anything else after the docstring.
    Update and correct the pre-existing docstring. Wrap everything to 88 chars.
    NEVER write back the initial code, JUST the docstring itself.
    Here is the task:

    <task>"#.to_owned();
    let post_prompt: String = r#"
</task>
    Assistant:

"#
    .to_owned();
    format!("{}\n{}\n{}", pre_prompt, function_text, post_prompt)
}

pub struct EnhanceClassDocstringProvider {
    prompt_handler: BedrockConverse,
}

impl EnhanceClassDocstringProvider {
    pub async fn new() -> Self {
        Self {
            prompt_handler: BedrockConverse::new().await.unwrap(),
        }
    }
}
#[async_trait]
impl ActionProvider for EnhanceClassDocstringProvider {
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
            let mut generated_docstring_text = self
                .prompt_handler
                .answer(&build_prompt(&function_text))
                .await
                .unwrap();

            let docstring_node = doc.find_first(&function_node, DOCSTRING_TS_QUERY);
            let target_range = if let Some(docstring_node) = docstring_node {
                generated_docstring_text = helper::indent_text(
                    &generated_docstring_text,
                    docstring_node.range().start_point.column,
                );
                // To keep correct indentation.
                let mut r = helper::ts_node_to_lsp_range(&docstring_node);
                r.start.character = 0;
                r
            } else {
                let function_body_node = doc.find_first(&function_node, BODY_TS_QUERY).unwrap();

                generated_docstring_text = format!(
                    "{}\n",
                    helper::indent_text(
                        &generated_docstring_text,
                        function_body_node.range().start_point.column
                    )
                );
                helper::prepend_ts_node_to_lsp_range(&function_body_node)
            };

            // for line in generated_docstring_text.lines() {
            //     println!("line {}", line);
            // }

            let text_edit = TextEdit {
                range: target_range,
                new_text: generated_docstring_text,
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
        let class_node = findup(cursor_node, "class_definition");

        if let Some(class_node) = class_node {
            let class_range = helper::ts_node_to_lsp_range(&class_node);

            return Some(CodeAction {
                title: "Update Class Docstring".to_string(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE),
                data: Some(json!(ResolveAction {
                    kind: ENHANCE_FUNCTION_DOCSTRING_ACTION.to_string(),
                    data: ActionContext {
                        uri: doc.uri.to_owned(),
                        ctx_range: class_range
                    }
                })),
                ..Default::default()
            });
        }
        None
    }
}
