# Polyglot-LS

Polyglot-LS is a language server that uses lua scripts or YAML configs to
create code-actions. It has access to context-aware information using
tree-sitter. These code actions are performed by a Large Language
Model (LLM) according to the prompt produced generated for the code action.

![Polyglot-LS Preview](.github/preview.png)

## Setup

### Prerequisites

- **Rust**: Ensure you have Rust installed. If not, install it from [the
  official Rust website](https://www.rust-lang.org/tools/install).
- **AWS Profile**: Create an AWS profile named `my-aws-bedrock` to get the
  correct credentials for using the Bedrock
  `anthropic.claude-3-haiku-20240307-v1:0` model.

### Compilation

To compile the project, follow these steps:

1. Clone the repository:

   ```sh
   git clone https://github.com/patwie/polyglot_ls.git
   cd polyglot_ls
   ```

2. Build the project:

   ```sh
   cargo build --release
   ```

3. The binary will be located in `target/release/polyglot_ls`.

### Using the Language Server

1. Copy the contents of the `code_actions` configs directory to
   `$HOME/.config/polyglot_ls/code_actions/`.

2. To run the server, execute:

   ```sh
   ./target/release/polyglot_ls
   ```

   For debugging, use:

   ```sh
   ./target/release/polyglot_ls --listen
   ```

   For direct usage in Neovim, use:

   ```sh
   ./target/release/polyglot_ls --stdin
   ```

## Limitations

The following are not hard limitations, but rather practical choices:

- This project only supports the AWS Bedrock model (no ChatGPT, no Ollama).
- Many settings are hard-coded (e.g., the used model).
- Only example code actions are implemented for Python and Rust.
- During compilation, Clippy will report some warnings in your terminal. This
  is a work-in-progress Proof of Concept.
- The name of this project may change in the future.

## Integration

For Neovim and the "neovim/nvim-lspconfig" plugin, use the following setup:

```lua
local configs = require 'lspconfig.configs'

if not configs.polyglot_ls then
    configs.polyglot_ls = {
      default_config = {
        cmd = { "/path/to/polyglot_ls" , "--stdin" },
        -- for debugging, launch "polyglot_ls" with --listen and use:
        -- cmd = vim.lsp.rpc.connect('127.0.0.1', 9257),
        filetypes = { 'python', 'rust' },
        single_file_support = true,
      },
    }
end
```

## Configuration Tutorial

See the [Tutorial.md](./TUTORIAL.md).
