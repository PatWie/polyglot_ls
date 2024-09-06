use clap::{ArgGroup, Parser};
use code_action_providers::load_providers;
use code_action_providers::parsed_document::ParsedDocument;
use llm_handlers::bedrock::BedrockConverse;
use llm_handlers::mock::MockLLM;
use llm_handlers::traits::Llm;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::{env, io};

pub mod code_action_providers;
pub mod configuration;
pub mod llm_handlers;
pub mod nonsense;
pub mod server;

use tokio::net::{TcpListener, TcpStream};
use tower_lsp::lsp_types::*;
use tower_lsp::{LspService, Server};

/// Reads all language configuration files in the specified directory that
/// match the given filter.
///
/// # Arguments
///
/// * `config_dir` - The directory containing the configuration files.
/// * `filter` - The file extension filter to apply.
///
/// # Returns
///
/// A vector of `PathBuf` containing the paths of the matching configuration files.
fn read_language_config_files(config_dir: &Path, filter: &str) -> Vec<PathBuf> {
    let mut config_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if (path.is_file() || path.is_symlink())
                && path.extension().map(|ext| ext == filter).unwrap_or(false)
            {
                config_files.push(path);
            }
        }
    }
    config_files
}
// https://github.com/microsoft/vscode-languageserver-node/blob/0cb3812e7d540ef3a904e96df795bc37a21de9b0/client/src/node/main.ts#L378-L387
#[derive(Parser)]
#[command(
    name = "polyglot_ls",
    version = "1.0",
    about = "An LLM-based lsp with lua scription and tree-sitter context"
)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(&["socket", "stdio", "bind", "answer"]),
))]
struct Args {
    /// Socket the LSP server will listen on
    #[arg(long)]
    socket: Option<u16>, // Option to allow it to be optional

    /// Socket the LSP server will bind on
    #[arg(long)]
    bind: Option<u16>, // Option to allow it to be optional

    /// LSP server will read input from stdin and reply in stdout
    #[arg(long)]
    stdio: bool, // Just a flag no value needed

    /// LSP server will read prompt from stdin and reply in stdout
    #[arg(long)]
    answer: bool, // Just a flag no value needed

    /// Will consume text from stdin, code_action id and cursor position from the arg
    /// and return the alternated text to stdout.
    #[arg(long)]
    use_mock: bool,

    /// Path to polyglot configuration YAML file
    #[arg(long)]
    polyglot_config_path: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    let home_dir = env::var("HOME").expect("Failed to get home directory");
    let config_base_dir = Path::new(&home_dir).join(".config").join("polyglot_ls");

    let polyglot_config_path = args.polyglot_config_path.unwrap_or(
        config_base_dir
            .join("server_config.yaml")
            .to_string_lossy()
            .to_string(),
    );
    let polyglot_config = configuration::PolyglotConfig::try_read_from_file(&polyglot_config_path)
        .unwrap_or_default();

    if args.stdio || args.answer {
    } else {
        tracing_subscriber::fmt::init();
    }
    //log::info!("Start");
    let prompt_handler;

    if args.use_mock {
        prompt_handler = Arc::new(Llm::Mock(MockLLM {
            answer: "MOCK".to_string(),
        }));
    } else {
        prompt_handler = Arc::new(Llm::Bedrock(
            BedrockConverse::new(&polyglot_config.model.bedrock)
                .await
                .unwrap(),
        ));
    }
    if args.answer {
        let mut prompt = String::new();
        io::stdin()
            .read_line(&mut prompt)
            .expect("Failed to read from stdin");
        let result = prompt_handler.answer(&prompt).await;
        if let Ok(answer) = result {
            println!("{}", &answer);
        } else {
            println!("{:?}", &result);
        }
        return;
    }

    let providers = load_providers(config_base_dir.join("code_actions"), prompt_handler);

    let (service, socket) = LspService::new(|client| server::Backend {
        client,
        current_text: Arc::new("".to_string().into()),
        current_language: Arc::new("".to_string().into()),
        providers,
        parsed_doc: ParsedDocument::new("", &Url::parse("http://example.com").unwrap(), ""),
        indexed_text: Arc::new(RwLock::new(nonsense::IndexedText::new("".to_owned()))),
    });

    if let Some(port) = args.socket {
        // If no argument is supplied (args is just the program name), then
        // we presume that the client has opened the TCP port and is waiting
        // for us to connect. This is the connection pattern used by clients
        // built with vscode-langaugeclient.
        let stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();

        let (read, write) = tokio::io::split(stream);
        #[cfg(feature = "runtime-agnostic")]
        let (read, write) = (read.compat(), write.compat_write());

        Server::new(read, write, socket).serve(service).await;
    } else if args.stdio {
        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
        #[cfg(feature = "runtime-agnostic")]
        let (stdin, stdout) = (stdin.compat(), stdout.compat_write());
        Server::new(stdin, stdout, socket).serve(service).await;
    } else if let Some(port) = args.bind {
        // If the `--bind` argument is supplied, then the roles are
        // reversed: we need to start a server and wait for the client to
        // connect.
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let (read, write) = tokio::io::split(stream);
        #[cfg(feature = "runtime-agnostic")]
        let (read, write) = (read.compat(), write.compat_write());

        Server::new(read, write, socket).serve(service).await;
    } else {
        println!("No input method specified");
    }
}
