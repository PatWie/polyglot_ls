# LLM-Sitter-LS

A single-binary language-server that uses to the tree-sitter parser to provide
context-aware code-actions. Those code-actions are performed by an
LLM.

![](.docs/preview.png)

## Setup

### Prerequisites

- **Rust**: Ensure you have Rust installed. If not, install it from [here](https://www.rust-lang.org/tools/install).
- **AWS Profile**: Create an AWS profile named `my-aws-bedrock` to get the correct credentials for using Bedrock `anthropic.claude-3-haiku-20240307-v1:0`.

### Compilation

To compile the project, follow these steps:

1. Clone the repository:

   ```sh
   git clone https://github.com/patwie/llm-sitter-ls.git
   cd llm-sitter-ls
   ```

2. Build the project:

   ```sh
   cargo build --release
   ```

3. The binary will be located in `target/release/llm-sitter-ls`.

### Running the Server

To run the server, execute:

```sh
./target/release/llm-sitter-ls
# For debugging
./target/release/llm-sitter-ls --list
# For direct usage in nvim
./target/release/llm-sitter-ls --stdin
```

## Limitations

These are not hard limitation per se but more a practical choice:

- This only supports AWS bedrock (no ChatGPT, no Ollama).
- There are many hard-coded setting (used model).
- Only example code-actions are implemented for Python.
- During compilation clippy will yell at you in your terminal. (This is a
  working Poc.)
- The name of this project might change.

## Integration

For Neovim and "neovim/nvim-lspconfig," use the following setup:

```lua
local configs = require 'lspconfig.configs'

if not configs.llmls then
    configs.llmls = {
      default_config = {
        cmd = { "/path/to/llm-sitter-ls" },
        -- for debugging, launch "llm-sitter-ls" with --listen and use:
        -- cmd = vim.lsp.rpc.connect('127.0.0.1', 9257),
        filetypes = { 'python' },
        single_file_support = true,
      },
    }
end
```
