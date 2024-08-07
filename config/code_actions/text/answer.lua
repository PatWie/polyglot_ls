local M = {
  is_triggered = function(selection_range)
    return true
  end,

  action_name = function()
    return "Answer"
  end,

  process_answer = function(text, selection_range)
    return "\n" .. text
  end,

  create_prompt = function(selection_range)
    local prompt_text = active_doc:text_from_range(selection_range)
    return prompt_text
  end,

  placement_range = function(selection_range)
    -- place after
    return {
      start_line = selection_range.end_line,
      start_character = 0,
      end_line = selection_range.end_line,
      end_character = 0,
    }
  end
}

return M
