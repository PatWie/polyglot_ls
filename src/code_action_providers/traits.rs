use serde::{Deserialize, Serialize};
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{CodeAction, Url},
};

use super::parsed_document::ParsedDocument;
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionContext {
    pub uri: Url,
    pub ctx_range: tower_lsp::lsp_types::Range,
}

#[async_trait]
pub trait ActionProvider: Send + Sync {
    fn can_handle(&self, action_name: &str) -> bool;
    async fn on_resolve(&self, doc: &ParsedDocument, action: CodeAction) -> Result<CodeAction>;
    fn create_code_action(
        &self,
        doc: &ParsedDocument,
        start_range: &tower_lsp::lsp_types::Range,
    ) -> Option<tower_lsp::lsp_types::CodeAction>;
}
