local M = {
  is_triggered = function(selection_range)
    return true
  end,

  action_name = function()
    return "Improve Wording"
  end,

  process_answer = function(text, selection_range)
    return text
  end,

  create_prompt = function(selection_range)
    local prompt_text = active_doc:text_from_range(selection_range)
    return table.concat({
      [=====[ Human:
      Improve the wording, fix grammar and typos. DO NOT output anything else. Just the improved text. No explanation.
]=====], prompt_text, [=====[
Assistant: ]=====] })
  end,

  placement_range = function(selection_range)
    return selection_range
  end
}

return M
