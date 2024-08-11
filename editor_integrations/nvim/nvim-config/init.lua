require("patwie")
function test_lsp_code_action(action_title)
  vim.defer_fn(function()
    vim.lsp.buf.code_action({
      filter = function(action)
        return action.title == action_title
      end,
      apply = true
    })
  end, 1000)   -- 1000 milliseconds = 1 second
end

