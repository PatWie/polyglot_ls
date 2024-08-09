local M = {
  is_triggered = function(lsp_range)
    return true
  end,

  action_name = function()
    return "Answer below"
  end,

  process_answer = function(llm_response, lsp_range)
    return llm_response
  end,

  create_prompt = function(lsp_range)
    local question_text = active_doc:text_from_range(lsp_range)
    local prompt = table.concat({
      [=====[ Human:
      You are a professional Assistant. Answer the given question. Your output can contain markdown. You output the assistant response.
      Here is the question:
<question> ]=====], question_text, [=====[
</question>
Assistant: ]=====] })
    print(prompt)
    return prompt
  end,

  placement_range = function(lsp_range)
    -- place below
    return {
      start_line = lsp_range.end_line + 1,
      start_character = 0,
      end_line = lsp_range.end_line + 1,
      end_character = 0,
    }
  end
}

return M
