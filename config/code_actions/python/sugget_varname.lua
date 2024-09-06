local function findup(node, kind)
  while node ~= nil and node:kind() ~= kind do
    node = node:parent()
  end
  return node
end

local M = {
  is_triggered = function(lsp_range)
    local start_node = active_doc:node_from_range(lsp_range)
    if start_node == nil then
      return nil
    end
    return findup(start_node, "identifier")
  end,

  action_name = function()
    return "List Identifier Name Options"
  end,

  process_answer = function(llm_response, lsp_range)
    return llm_response
  end,

  create_prompt = function(lsp_range)
    local start_node = active_doc:node_from_range(lsp_range)
    if start_node == nil then
      return nil
    end
    local var_node = findup(start_node, "identifier")
    local var_text = active_doc:text_from_node(var_node)
    local func_node = findup(start_node, "function_definition")
    local function_text = active_doc:text_from_node(func_node)

    return table.concat({
      [=====[ Human:
      The variable <variable>]=====], var_text, [=====[</variable>
      is used in the following context

      <context>
]=====], function_text, [=====[
      </context>

      List me 20 Options of how I can name that identifer given the context.
Assistant: ]=====] })
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
