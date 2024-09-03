local prompt = [[

Please take the following unstructured draft text, which may contain typos and unclear phrasing,
and transform it into a clear, concise, and well-structured Git commit message according
to the following rules:

<instructions>
1. Separate subject from body with a blank line
2. Limit the subject line to 50 characters
3. Capitalize the subject line
4. Do not end the subject line with a period
5. Use the imperative mood in the subject line
6. Wrap the body at 72 characters
7. Use the body to explain what and why vs. how
</instructions>



---
]]

local M = {
  is_triggered = function(lsp_range)
    return true
  end,

  action_name = function()
    return "Improve Git Message"
  end,

  process_answer = function(llm_response, lsp_range)
    return llm_response
  end,

  create_prompt = function(lsp_range)
    local git_draft_message = active_doc:text_from_range(active_doc:root():range())
    return table.concat({
      [[ Human:
]], prompt, [[

Here is the unstructured draft text:

<task>
]], git_draft_message, [[</task>
Assistant: ]] })
  end,

  placement_range = function(lsp_range)
    return active_doc:root():range()
  end
}

return M
