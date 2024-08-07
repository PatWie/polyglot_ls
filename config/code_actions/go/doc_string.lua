local function findup(node, kind)
  while node ~= nil and node:kind() ~= kind do
    node = node:parent()
  end
  return node
end

local function extract_docstring(fn_node)
  local docstring = ""
  local node = fn_node:prev_sibling()
  while node ~= nil and node:kind() == "comment" do
    docstring = active_doc:text_from_node(node) .. docstring
    node = node:prev_sibling()
  end

  return docstring
end

return {
  is_triggered = function(selection_range)
    local cursor_node = active_doc:node_from_range(selection_range)
    if cursor_node == nil then
      return false
    end
    return findup(cursor_node, "function_declaration") ~= nil
  end,

  action_name = function()
    return "Update Docstring"
  end,

  process_answer = function(text, selection_range)
    return text .. "\n"
  end,

  create_prompt = function(selection_range)
    local cursor_node = active_doc:node_from_range(selection_range)
    if cursor_node == nil then
      return nil
    end
    local fn_node = findup(cursor_node, "function_declaration")
    if fn_node ~= nil then
      local hint = active_doc:text_from_node(fn_node)

      local doc_string = extract_docstring(fn_node)

      return table.concat({
        [=====[
Human: Write a golang docstring for a given function. Follow the style of

// splitExampleName attempts to split example name s at index i,
// and reports if that produces a valid split. The suffix may be
// absent. Otherwise, it must start with a lower-case letter and
// be preceded by '_'.
//
// One of i == len(s) or s[i] == '_' must be true.
func splitExampleName(s string, i int) (prefix, suffix string, ok bool) {
    if i == len(s) {
        return s, "", true
    }
    if i == len(s)-1 {
        return "", "", false
    }
    prefix, suffix = s[:i], s[i+1:]
    return prefix, suffix, isExampleSuffix(suffix)
}

NEVER write anything else besides the docstring block. ONLY generate the docstring,
It should include Arguments, Returns, Example if necessary. First line must be in imperative mood. Do NOT output anything else after the docstring.
Update and correct the pre-existing docstring, parametern names or types might have changed. Wrap everything to 88 chars.
NEVER write back the initial code, JUST the docstring itself.

Here is the task:
<task> ]=====], doc_string, hint, [=====[
</task>
Assistant: ]=====] })
    end
  end,

  placement_range = function(selection_range)
    local start_node = active_doc:node_from_range(selection_range)
    if start_node == nil then
      return nil
    end
    local fn_node = findup(start_node, "function_declaration")
    if fn_node ~= nil then
      local range = fn_node:range()
      -- before the function
      local ret = {
        start_line = range.start_line,
        start_character = 0,
        end_line = range.start_line,
        end_character = 0,
      }

      -- collect docstrings
      local p_node = fn_node:prev_sibling()
      while p_node ~= nil and p_node:kind() == "comment" do
        local range = p_node:range()
        ret.start_line = range.start_line
        p_node = p_node:prev_sibling()
      end
      return ret
    end
  end
}
