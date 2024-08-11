use crate::code_action_providers::{find_resolver, map_to_lsp};
use crate::nonsense::{self, TextAdapter};

use super::code_action_providers::parsed_document::ParsedDocument;
use super::code_action_providers::traits::ActionProvider;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tower_lsp::jsonrpc::{self, Result};
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveActionKind {
    pub id: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveAction<T> {
    /// The data to be resolved.
    pub data: T,
    /// The unique identifier for this resolve action.
    pub id: String,
}

pub(crate) struct Backend {
    /// The client used for communicating with the backend.
    pub client: Client,
    /// The current text being processed.
    pub current_text: Arc<RwLock<String>>,
    /// The current language being processed.
    pub current_language: Arc<RwLock<String>>,
    /// A map of action providers, keyed by the name of the provider.
    pub providers: HashMap<String, Vec<Box<dyn ActionProvider>>>,
    /// The parsed document being processed.
    pub parsed_doc: ParsedDocument,

    pub indexed_text: Arc<RwLock<nonsense::IndexedText<String>>>,
    // translation: Translation,
}

impl std::fmt::Debug for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Backend")
            .field("client", &self.client)
            .field("current_text", &self.current_text)
            .finish()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "PolyglotLS".to_string(),
                version: None,
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                code_action_provider: Some(CodeActionProviderCapability::Options(
                    CodeActionOptions {
                        code_action_kinds: Some(vec![
                            CodeActionKind::QUICKFIX,
                            CodeActionKind::REFACTOR_INLINE,
                            CodeActionKind::REFACTOR_REWRITE,
                            CodeActionKind::REFACTOR,
                        ]),
                        resolve_provider: Some(true),
                        work_done_progress_options: WorkDoneProgressOptions {
                            work_done_progress: None,
                        },
                    },
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    /// Initialize the language server.
    ///
    /// # Arguments
    ///
    /// * `_: InitializedParams` - The initialization parameters.
    ///
    /// # Returns
    ///
    /// None.
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    /// Resolve a code action by delegating to language-specific providers.
    ///
    /// # Arguments
    ///
    /// * `action` - The code action to resolve.
    ///
    /// # Returns
    ///
    /// The resolved code action.
    async fn code_action_resolve(&self, action: CodeAction) -> Result<CodeAction> {
        //log::info!("code_action_resolve {:?}", action);
        let json_args = action.data.clone().unwrap();
        let args = serde_json::from_value::<ResolveActionKind>(json_args.clone()).unwrap();

        let source = self.current_text.read().unwrap().clone();
        let lang = self.current_language.read().unwrap().clone();
        let parsed_doc = ParsedDocument::new(&source, &self.parsed_doc.uri, &lang);
        let index = self.indexed_text.read().unwrap().clone();

        let provider = find_resolver(&self.providers, &args.id, &lang);
        if provider.is_none() {
            return Err(jsonrpc::Error::new(jsonrpc::ErrorCode::ServerError(1)));
        }

        let code_action = provider
            .unwrap()
            .on_resolve(&parsed_doc, action.clone())
            .await;

        match code_action {
            Ok(mut c) => {
                map_to_lsp(&mut c, &index);
                Ok(c)
            }
            Err(_) => Err(jsonrpc::Error::new(jsonrpc::ErrorCode::ServerError(1))),
        }
    }

    /// Provide code actions for the current document.
    ///
    /// # Arguments
    ///
    /// * `params` - The code action parameters.
    ///
    /// # Returns
    ///
    /// A `Result<Option<CodeActionResponse>>` containing the code actions or an error.
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.client
            .log_message(MessageType::INFO, "code action")
            .await;

        let uri = &params.text_document.uri;
        let source = self.current_text.read().unwrap().clone();
        let lang = self.current_language.read().unwrap().clone();
        let index = self.indexed_text.read().unwrap().clone();
        let doc = ParsedDocument::new(&source, uri, &lang);

        // LSP is UTF16, our abckend is UTF8
        let lsp_range = params.range;
        let rs = index.lsp_range_to_range(&lsp_range).unwrap();
        let fake_lsp_range = Range {
            start: Position {
                line: rs.start.line,
                character: rs.start.col,
            },
            end: Position {
                line: rs.end.line,
                character: rs.end.col,
            },
        };

        let mut actions = vec![];
        if let Some(language_specific_providers) = self.providers.get(&lang) {
            for provider in language_specific_providers.iter() {
                if let Some(action) = provider.create_code_action(&doc, &fake_lsp_range) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }
        }
        if let Some(language_specific_providers) = self.providers.get("__all__") {
            for provider in language_specific_providers.iter() {
                if let Some(action) = provider.create_code_action(&doc, &fake_lsp_range) {
                    actions.push(CodeActionOrCommand::CodeAction(action));
                }
            }
        }

        Ok(Some(actions))
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    /// Handle a text document open notification.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters for the text document open notification.
    ///
    /// # Returns
    ///
    /// This function does not return a value.
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        //log::info!("did_open");
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        let mut src = self.current_text.write().unwrap();
        *src = params.text_document.text.clone();
        let mut src = self.current_language.write().unwrap();
        *src = params.text_document.language_id.clone();

        let mut src = self.indexed_text.write().unwrap();
        *src = nonsense::IndexedText::new(params.text_document.text.to_owned());
        //log::info!("set language to {}", &params.text_document.language_id);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        let mut src = self.current_text.write().unwrap();
        *src = params.content_changes[0].text.clone();
        let mut src = self.indexed_text.write().unwrap();
        *src = nonsense::IndexedText::new(params.content_changes[0].text.to_owned());
    }
    //
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
        if let Some(new_text) = params.text {
            let mut src = self.current_text.write().unwrap();
            *src = new_text.clone();
            let mut src = self.indexed_text.write().unwrap();
            *src = nonsense::IndexedText::new(new_text);
        }
    }
    //
    // async fn did_close(&self, _: DidCloseTextDocumentParams) {
    //     self.client
    //         .log_message(MessageType::INFO, "file closed!")
    //         .await;
    // }
    //
    // async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
    //     Ok(Some(CompletionResponse::Array(vec![
    //         CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
    //         CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
    //     ])))
    // }
}
