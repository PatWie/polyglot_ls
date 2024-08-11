# Polyglot-LS

The only sensible way to bring Large Language Models (LLMs) into any editor
that speaks the Language Server Protocol (LSP).

![Polyglot-LS Preview](.github/preview.gif)

Polyglot-LS is a language server written in Rust that embeds Lua scripting,
responsible for creating prompts, post-processing LLM answers, and determining
where to place the LLM output. It has access to the Tree-Sitter Abstract Syntax
Tree (AST) and can navigate nodes and form Tree-Sitter queries.

![Polyglot-LS Overview](.github/polyglot-ls-overview-scene.svg)

## Some Use Cases

Utilizing Lua scripting allows you to access structured information from the
current file and construct any string for LLM prompts. Here are some practical
use cases:

### Add Documentation

Automatically add or update docstrings for functions in various languages. This
provides context and determines where to place these docstrings, enabling
documentation updates without altering the code.

### Adjust Function Signatures

Using tree-sitter, a code-action can extract context from various parts of your
code, such as the function source, the nearest class source, or any specific
Tree-Sitter node. This allows you to form prompts that target other Tree-Sitter
nodes, such as parameter lists of functions or other code segments, which can
then be substituted with LLM outputs. This provides you with full control over
which parts of the code remain untouched, ensuring precise and targeted
modifications.

### Chat Locally

Engage with LLMs using Markdown files. When you select your prompt, all
previous text is used as context, and the model's response is output below,
similar to web-based UIs.

For Neovim users, combine this with fuzzy-finding of all `.md` files in a
specific directory:

```lua
vim.keymap.set("n", "<leader>cc", function()
  builtin.find_files({
    prompt_title = "< Chats >",
    cwd = "$HOME/.chats/",
  })
end)
```

Pair this with [undotree](https://github.com/mbbill/undotree) for powerful
history features and quick access to different chats.

### Fix Typos

Use tree-sitter information to fix selected text wording, including Git commit
messages written in your editor. Or even reformulate entire Git commit
messages.

See [configs](./config/code_actions/) for examples.

## Setup

### Prerequisites

- **Rust**: Install Rust from the [official Rust
  website](https://www.rust-lang.org/tools/install).
- **AWS Profile**: Create an AWS profile named `my-aws-bedrock` to obtain
  credentials for using the Bedrock `anthropic.claude-3-haiku-20240307-v1:0`
  model.

### Compilation

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

1. Copy or symlink the contents of the `code_actions` configs directory to
   `$HOME/.config/polyglot_ls/code_actions/`.

    ```sh
    mkdir -p ${HOME}/.config/polyglot_ls
    ln -s $(realpath config/code_actions) ${HOME}/.config/polyglot_ls/code_actions
    ```

2. Run the server:

   ```sh
   ./target/release/polyglot_ls --socket=9257
   ```

   For direct usage in Neovim:

   ```sh
   ./target/release/polyglot_ls --stdio
   ```

## Limitations

- Currently supports only the AWS Bedrock model (no ChatGPT, no Ollama).
- Many settings are hard-coded (e.g., the used model).

## Integration

For Neovim with the "neovim/nvim-lspconfig" plugin, use the following setup:

```lua
local configs = require 'lspconfig.configs'

if not configs.polyglot_ls then
    configs.polyglot_ls = {
      default_config = {
        cmd = { "/path/to/polyglot_ls" , "--stdio" },
        -- for debugging, launch "polyglot_ls" with --bind=9257 and use:
        -- cmd = vim.lsp.rpc.connect('127.0.0.1', 9257),
        filetypes = { 'python', 'rust', 'text', 'go', 'gitcommit', 'markdown' },
        single_file_support = true,
      },
    }
end
```

## Configuration Tutorial

See the [Tutorial.md](./TUTORIAL.md).

# Test Integration

Prepare NVIM integration tests via

```sh
ln -s $(realpath editor_integrations/nvim/nvim-config) ${HOME}/.config/nvim-test
NVIM_APPNAME=nvim-test nvim --headless "+Lazy! update" +qa
```

Then running

```sh
NVIM_APPNAME=nvim-test ./tests/run_all.sh
```
