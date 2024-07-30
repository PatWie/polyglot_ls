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

pub const ENHANCE_FUNCTION_DOCSTRING_ACTION: &str = "povider.py.func_docstring";

const DOCSTRING_TS_QUERY: &str = r#"
(function_definition
  body: (block
    (expression_statement
      (string) @docstring)))
"#;
const BODY_TS_QUERY: &str = r#"
(function_definition
  body: (block) @body)
"#;

fn build_prompt(function_text: &str) -> String {
    let pre_prompt: String = r#"
Human: Write a google style docstring for a given function. Here is an example
    for

    def fetch_smalltable_rows(
        table_handle: smalltable.Table,
        keys: Sequence[bytes | str],
        require_all_keys: bool = False,
    ) -> Mapping[bytes, tuple[str, ...]]:

    how it can look like

        """Fetch rows from a Smalltable.

        Retrieves rows pertaining to the given keys from the Table instance
        represented by table_handle.  String keys will be UTF-8 encoded.

        Args:
            table_handle: An open smalltable.Table instance.
            keys: A sequence of strings representing the key of each table
              row to fetch.  String keys will be UTF-8 encoded.
            require_all_keys: If True only rows with values set for all keys will be
              returned.

        Returns:
            A dict mapping keys to the corresponding table row data
            fetched. Each row is represented as a tuple of strings. For
            example:

            {b'Serak': ('Rigel VII', 'Preparer'),
             b'Zim': ('Irk', 'Invader'),
             b'Lrrr': ('Omicron Persei 8', 'Emperor')}

            Returned keys are always bytes.  If a key from the keys argument is
            missing from the dictionary, then that row was not found in the
            table (and require_all_keys must have been False).

        Raises:
            IOError: An error occurred accessing the smalltable.

        Examples:
            >>> my_table = fetch_smalltable_rows(handle, ["id", "user"], True)
        """

    NEVER write anything else besides the docstring block. ONLY generate the docstring,
    It should include Args, Returns, Raise, Yield, Attributes, Notes, Example if necessary. First line must be in imperative mood. Do NOT output anything else after the docstring.
    Update and correct the pre-existing docstring, parametern names or types might have changed. Wrap everything to 88 chars.
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

pub struct EnhanceFunctionDocstringProvider {
    prompt_handler: BedrockConverse,
}

impl EnhanceFunctionDocstringProvider {
    pub async fn new() -> Self {
        Self {
            prompt_handler: BedrockConverse::new().await.unwrap(),
        }
    }
}
#[async_trait]
impl ActionProvider for EnhanceFunctionDocstringProvider {
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
        let function_node = findup(cursor_node, "function_definition");

        if let Some(function_node) = function_node {
            let function_range = helper::ts_node_to_lsp_range(&function_node);

            return Some(CodeAction {
                title: "Update Function Docstring".to_string(),
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
