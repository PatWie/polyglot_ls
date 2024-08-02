use code_action_providers::config;
use code_action_providers::generic::GenericProvider;
use code_action_providers::parsed_document::ParsedDocument;
use code_action_providers::traits::ActionProvider;
use prompt_handlers::claude::BedrockConverse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub mod code_action_providers;
pub mod prompt_handlers;

use tokio::net::{TcpListener, TcpStream};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

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

struct Backend {
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
                name: "LlmSitterLs".to_string(),
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
        log::info!("code_action_resolve {:?}", action);
        let json_args = action.data.clone().unwrap();
        let args = serde_json::from_value::<ResolveActionKind>(json_args.clone()).unwrap();

        let source = self.current_text.read().unwrap().clone();
        let lang = self.current_language.read().unwrap().clone();
        let parsed_doc = ParsedDocument::new(&source, &self.parsed_doc.uri, &lang);

        if let Some(language_specific_providers) = self.providers.get(&lang) {
            for provider in language_specific_providers.iter() {
                if provider.can_handle(args.id.as_str()) {
                    return provider.on_resolve(&parsed_doc, action.clone()).await;
                }
            }
        }
        todo!();
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
        let doc = ParsedDocument::new(&source, uri, &lang);

        let mut actions = vec![];
        if let Some(language_specific_providers) = self.providers.get(&lang) {
            for provider in language_specific_providers.iter() {
                if let Some(action) = provider.create_code_action(&doc, &params.range) {
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
        log::info!("did_open");
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        let mut src = self.current_text.write().unwrap();
        *src = params.text_document.text.replace("\\n", "\n");
        let mut src = self.current_language.write().unwrap();
        *src = params.text_document.language_id.clone();
        log::info!("set language to {}", &params.text_document.language_id);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        let mut src = self.current_text.write().unwrap();
        *src = params.content_changes[0].text.replace("\\n", "\n")
    }
    //
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
        if let Some(new_text) = params.text {
            let mut src = self.current_text.write().unwrap();
            *src = new_text.replace("\\n", "\n");
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

fn read_language_config_files(config_dir: &Path) -> Vec<String> {
    let mut config_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if (path.is_file() || path.is_symlink())
                && path.extension().map(|ext| ext == "yaml").unwrap_or(false)
            {
                if let Some(file_stem) = path.file_stem() {
                    if let Some(file_name) = file_stem.to_str() {
                        config_files.push(file_name.to_string());
                    }
                }
            }
        }
    }
    config_files
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    tracing_subscriber::fmt().init();
    log::info!("Start");
    let prompt_handler = Arc::new(BedrockConverse::new().await.unwrap());
    let mut providers: HashMap<String, Vec<Box<dyn ActionProvider>>> = Default::default();

    let home_dir = env::var("HOME").expect("Failed to get home directory");
    let config_dir = Path::new(&home_dir)
        .join(".config")
        .join("llm-sitter-ls")
        .join("code_actions");

    log::info!("Processing  config-dir: {:?}", config_dir);
    for language in read_language_config_files(&config_dir) {
        log::info!("Processing language config: {}", language);
        let path = Path::new(&home_dir)
            .join(".config")
            .join("llm-sitter-ls")
            .join("code_actions")
            .join(format!("{}.yaml", language));
        match config::CodeActionConfig::from_yaml(&path) {
            Ok(language_config) => {
                for (k, config) in language_config.code_actions.into_iter().enumerate() {
                    log::info!("Register action {} for {}", config.name, language);
                    providers
                        .entry(language.clone())
                        .or_default()
                        .push(Box::new(GenericProvider::from_config(
                            config,
                            &format!("py.{k}"),
                            prompt_handler.clone(),
                        )));
                }
            }
            Err(e) => log::warn!("Cannot read {:?} because of {}", &path, e),
        };
    }

    let (service, socket) = LspService::new(|client| Backend {
        client,
        current_text: Arc::new("".to_string().into()),
        current_language: Arc::new("".to_string().into()),
        providers,
        parsed_doc: ParsedDocument::new("", &Url::parse("http://example.com").unwrap(), ""),
    });

    let mut args = std::env::args();
    match args.nth(1).as_deref() {
        None => {
            // If no argument is supplied (args is just the program name), then
            // we presume that the client has opened the TCP port and is waiting
            // for us to connect. This is the connection pattern used by clients
            // built with vscode-langaugeclient.
            let stream = TcpStream::connect("127.0.0.1:9257").await.unwrap();
            let (read, write) = tokio::io::split(stream);
            #[cfg(feature = "runtime-agnostic")]
            let (read, write) = (read.compat(), write.compat_write());

            Server::new(read, write, socket).serve(service).await;
        }
        Some("--stdin") => {
            let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
            #[cfg(feature = "runtime-agnostic")]
            let (stdin, stdout) = (stdin.compat(), stdout.compat_write());

            Server::new(stdin, stdout, socket).serve(service).await;
        }
        Some("--listen") => {
            // If the `--listen` argument is supplied, then the roles are
            // reversed: we need to start a server and wait for the client to
            // connect.
            let listener = TcpListener::bind("127.0.0.1:9257").await.unwrap();
            let (stream, _) = listener.accept().await.unwrap();
            let (read, write) = tokio::io::split(stream);
            #[cfg(feature = "runtime-agnostic")]
            let (read, write) = (read.compat(), write.compat_write());

            Server::new(read, write, socket).serve(service).await;
        }
        Some(arg) => panic!(
            "Unrecognized argument: {}. Use --listen to listen for connections or --stdin to use stdin.",
            arg
        ),
    };
}
