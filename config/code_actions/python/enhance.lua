local M = {
  is_triggered = function(lsp_range)
    return true
  end,

  action_name = function()
    return "Improve Code"
  end,

  process_answer = function(llm_response, lsp_range)
    return llm_response
  end,

  create_prompt = function(lsp_range)
    print("in range")
    print(lsp_range)
    local function_text = active_doc:text_from_range(lsp_range)

    return table.concat({
      [=====[ Human:
      You are a professional python3 coder. You got a given code which might
      contain bugs is badly written and has bad naming convention.

      Improve the code by using concise variable names, fix logic bugs, rearange the
      code if necessary, split if required. But do not overdo things. Follow pep8
      guidelines for naming and everything else. If there is a sublte bug, fix the bug.

      Just output the final enhanced code without any explanation.

      Here is the task:
<task> ]=====], function_text, [=====[
</task>
Assistant: ]=====] })
  end,

  placement_range = function(lsp_range)
    return lsp_range
  end
}
return M
