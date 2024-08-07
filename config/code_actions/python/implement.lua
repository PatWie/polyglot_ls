local M = {
  is_triggered = function(selection_range)
    return true
  end,

  action_name = function()
    return "Write Code"
  end,

  process_answer = function(text, selection_range)
    return text
  end,

  create_prompt = function(selection_range)
    local prompt_text = active_doc:text_from_range(selection_range)


    return table.concat({
      [=====[ Human:
      You are a professional python3 coder. Implement what is requested from you.
      Just output the final enhanced code without any explanation. Do not output anything else.
      Here is the task:
<task> ]=====], prompt_text, [=====[
</task>
Assistant: ]=====] })
  end,

  placement_range = function(selection_range)
    return selection_range
  end
}

return M
