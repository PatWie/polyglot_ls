local function findup(node, kind)
  while node ~= nil and node:kind() ~= kind do
    node = node:parent()
  end
  return node
end



local function find_specific_node(start_range, kind)
  local start_node = active_doc:node_from_range(start_range)
  if start_node == nil then
    return nil
  end
  return findup(start_node, kind)
end


return {
  is_triggered = function(lsp_range)
    return find_specific_node(lsp_range, "function_definition") ~= nil
  end,

  action_name = function()
    return "Update Function Args Type Annotations"
  end,

  process_answer = function(llm_response, lsp_range)
    return "(" .. llm_response .. ")"
  end,

  create_prompt = function(lsp_range)
    local fn_node = find_specific_node(lsp_range, "function_definition")
    if fn_node ~= nil then
      local function_text = active_doc:text_from_node(fn_node)
      local args_node = active_doc:query_first(fn_node, [[(function_definition
            parameters: (parameters) @parameters)]])
      local args_text = active_doc:text_from_node(args_node) or ""


      return table.concat({
        [=====[ Human:
      Human: Enhance the function parameters by updating or adding python3 type annotations

      for
          def fetch_smalltable_rows(self, table_handle, keys,
              require_all_keys: bool = False,
          ):

      a version with annotations might look like

          def fetch_smalltable_rows(self,
              table_handle: smalltable.Table,
              keys: Sequence[bytes | str],
              require_all_keys: bool = False,
          ) -> Mapping[bytes, tuple[str, ...]]:

      In this case the output would be

              self,
              table_handle: smalltable.Table,
              keys: Sequence[bytes | str],
              require_all_keys: bool = False,

      Use the correct type by understand the function body. Do NOT use "Any" if you can derive the correct type from the function body.
      If there are pre-existing default values, keep them as they are oif they make sense.
      Remember, class methods start with "self" as first arguments without an annotation. Keep pre-existing "self" args.
      ONLY output the parameters comma-separated, without function name and parentheses

      Here is the function:
 ]=====], function_text, [=====[

 Here is the task:
<task> ]=====], args_text, [=====[
</task>
Assistant: ]=====] })
    end
  end,

  placement_range = function(lsp_range)
    local fn_node = find_specific_node(lsp_range, "function_definition")
    local args_node = active_doc:query_first(fn_node, [[(function_definition
            parameters: (parameters) @parameters)]])
    return args_node:range()
  end
}
