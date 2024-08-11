return {

  {
    "neovim/nvim-lspconfig",

    config = function()
      local util = require 'lspconfig.util'
      local configs = require 'lspconfig.configs'

      if not configs.polyglot_ls then
        configs.polyglot_ls = {
          default_config = {
            cmd = { "/tmp/polyglot_ls" },
            filetypes = { 'python', 'rust', 'text', 'go', 'gitcommit', 'markdown' },
            root_dir = util.root_pattern(unpack({
              'pyproject.toml',
              'ruff.toml',
              'Cargo.toml',
            })) or util.find_git_ancestor(),
            single_file_support = true,
          },
        }
      end

      local servers = {
        polyglot_ls = {
          -- cmd = vim.lsp.rpc.connect('127.0.0.1', 9257),
          cmd = { "/tmp/polyglot_ls", "--stdio" , "--use-mock" },
          filetypes = { 'python', 'rust', 'text', 'go', 'gitcommit', 'markdown' },
        },
      }

      local capabilities = vim.lsp.protocol.make_client_capabilities()

      for server_name, server_config in pairs(servers) do
        server_config.capabilities = capabilities
        server_config.flags = {
          debounce_text_changes = 200,
          allow_incremental_sync = true,
        }
      local lspconfig = require("lspconfig")
        lspconfig[server_name].setup(server_config)
      end
    end,
  },
}
