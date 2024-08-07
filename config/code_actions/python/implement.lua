local M = {
  is_triggered = function(lsp_range)
    return true
  end,

  action_name = function()
    return "Write Code"
  end,

  process_answer = function(llm_response, lsp_range)
    return llm_response
  end,

  create_prompt = function(lsp_range)
    local prompt_text = active_doc:text_from_range(lsp_range)


    return table.concat({
      [=====[ Human:
      You are a professional python3 coder. Implement what is requested from you.
      Just output the final enhanced code without any explanation. Do not output anything else.
      Here is the task:
<task> ]=====], prompt_text, [=====[
</task>
Assistant: ]=====] })
  end,

  placement_range = function(lsp_range)
    return lsp_range
  end
}

return M
