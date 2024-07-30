use code_action_providers::parsed_document::ParsedDocument;
use code_action_providers::traits::ActionProvider;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

pub mod code_action_providers;
pub mod prompt_handlers;

use code_action_providers::python::class_docstring::EnhanceClassDocstringProvider;
use code_action_providers::python::comment::EnhanceCommentProvider;
use code_action_providers::python::function_args::EnhanceFunctionArgsProvider;
use code_action_providers::python::function_docstring::EnhanceFunctionDocstringProvider;

use tokio::net::{TcpListener, TcpStream};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveActionKind {
    pub kind: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveAction<T> {
    pub data: T,
    pub kind: String,
}

struct Backend {
    client: Client,
    current_text: Arc<RwLock<String>>,
    providers: Vec<Box<dyn ActionProvider>>,
    parsed_doc: ParsedDocument,
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
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn code_action_resolve(&self, action: CodeAction) -> Result<CodeAction> {
        log::info!("code_action_resolve {:?}", action);
        let json_args = action.data.clone().unwrap();
        let args = serde_json::from_value::<ResolveActionKind>(json_args.clone()).unwrap();

        let source = self.current_text.read().unwrap().clone();
        let parsed_doc = ParsedDocument::new(&source, &self.parsed_doc.uri);

        for provider in self.providers.iter() {
            if provider.can_handle(args.kind.as_str()) {
                return provider.on_resolve(&parsed_doc, action.clone()).await;
            }
        }
        todo!();
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        self.client
            .log_message(MessageType::INFO, "code action")
            .await;

        let uri = &params.text_document.uri;
        let source = self.current_text.read().unwrap();
        let doc = ParsedDocument::new(&source, uri);

        let mut actions = vec![];
        for provider in self.providers.iter() {
            if let Some(action) = provider.create_code_action(&doc, &params.range) {
                actions.push(CodeActionOrCommand::CodeAction(action));
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        log::info!("did_open");
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        let mut src = self.current_text.write().unwrap();
        *src = params.text_document.text.replace("\\n", "\n")
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        log::info!("did_change");
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        let mut src = self.current_text.write().unwrap();
        *src = params.content_changes[0].text.replace("\\n", "\n")
    }
    //
    // async fn did_save(&self, _: DidSaveTextDocumentParams) {
    //     self.client
    //         .log_message(MessageType::INFO, "file saved!")
    //         .await;
    // }
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

#[tokio::main]
async fn main() {
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    tracing_subscriber::fmt().init();
    log::info!("Start");
    let p1 = EnhanceFunctionDocstringProvider::new().await;
    let p2 = EnhanceClassDocstringProvider::new().await;
    let p3 = EnhanceCommentProvider::new().await;
    let p4 = EnhanceFunctionArgsProvider::new().await;
    let (service, socket) = LspService::new(|client| Backend {
        client,
        current_text: Arc::new("".to_string().into()),
        providers: vec![Box::new(p1), Box::new(p2), Box::new(p3), Box::new(p4)],
        parsed_doc: ParsedDocument::new("", &Url::parse("http://example.com").unwrap()),
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
