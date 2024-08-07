local M = {
  is_triggered = function(lsp_range)
    return true
  end,

  action_name = function()
    return "Answer"
  end,

  process_answer = function(llm_response, lsp_range)
    return "\n## Assistant\n\n" .. llm_response .. "\n\n## Human \n\n"
  end,

  create_prompt = function(lsp_range)
    local previous_text = active_doc:text_from_range({
      start_line = 0,
      start_character = 0,
      end_line = lsp_range.start_line,
      end_character = 0,
    })
    local question_text = active_doc:text_from_range(lsp_range)
    local prompt = table.concat({
      [=====[ Human:
<context> ]=====], previous_text, [=====[ </context>
      You are a professional Assistant. Answer the given question. Your output can contain markdown. You output the assistant response.
      Here is the question:
<question> ]=====], question_text, [=====[
</question>
Assistant: ]=====] })
    print(prompt)
    return prompt
  end,

  placement_range = function(lsp_range)
    local doc_range = active_doc:root():range()
    -- place after
    return {
      start_line = lsp_range.end_line + 1,
      start_character = 0,
      end_line = doc_range.end_line,
      end_character = doc_range.end_character,
    }
  end
}

return M
