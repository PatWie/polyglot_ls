local function findup(node, kinds)
  while node ~= nil do
    for _, kind in ipairs(kinds) do
      if node:kind() == kind then
        return node
      end
    end
    node = node:parent()
  end
  return nil
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
local allowed_kinds = { "function_declaration", "method_declaration" }

return {
  is_triggered = function(lsp_range)
    local cursor_node = active_doc:node_from_range(lsp_range)
    if cursor_node == nil then
      return false
    end
    return (findup(cursor_node, allowed_kinds) ~= nil)
  end,

  action_name = function()
    return "Update Docstring"
  end,

  process_answer = function(llm_response, lsp_range)
    return llm_response .. "\n"
  end,

  create_prompt = function(lsp_range)
    local cursor_node = active_doc:node_from_range(lsp_range)
    if cursor_node == nil then
      return nil
    end
    local fn_node = findup(cursor_node, allowed_kinds)
    if fn_node ~= nil then
      local hint = active_doc:text_from_node(fn_node)

      local doc_string = extract_docstring(fn_node)

      -- https://cs.opensource.google/go/go/+/refs/tags/go1.23.0:src/sort/search.go;l=58
      return table.concat({
        [=====[
<instruction>Write a golang documentation for a given function. Follow the style of

<example>
<input>
func Find(n int, cmp func(int) int) (i int, found bool) {
	// The invariants here are similar to the ones in Search.
	// Define cmp(-1) > 0 and cmp(n) <= 0
	// Invariant: cmp(i-1) > 0, cmp(j) <= 0
	i, j := 0, n
	for i < j {
		h := int(uint(i+j) >> 1) // avoid overflow when computing h
		// i ≤ h < j
		if cmp(h) > 0 {
			i = h + 1 // preserves cmp(i-1) > 0
		} else {
			j = h // preserves cmp(j) <= 0
		}
	}
	// i == j, cmp(i-1) > 0 and cmp(j) <= 0
	return i, i < n && cmp(i) == 0
}
</input>
<output>
// Find uses binary search to find and return the smallest index i in [0, n)
// at which cmp(i) <= 0. If there is no such index i, Find returns i = n.
// The found result is true if i < n and cmp(i) == 0.
// Find calls cmp(i) only for i in the range [0, n).
//
// To permit binary search, Find requires that cmp(i) > 0 for a leading
// prefix of the range, cmp(i) == 0 in the middle, and cmp(i) < 0 for
// the final suffix of the range. (Each subrange could be empty.)
// The usual way to establish this condition is to interpret cmp(i)
// as a comparison of a desired target value t against entry i in an
// underlying indexed data structure x, returning <0, 0, and >0
// when t < x[i], t == x[i], and t > x[i], respectively.
//
// For example, to look for a particular string in a sorted, random-access
// list of strings:
//
//	i, found := sort.Find(x.Len(), func(i int) int {
//	    return strings.Compare(target, x.At(i))
//	})
//	if found {
//	    fmt.Printf("found %s at entry %d\n", target, i)
//	} else {
//	    fmt.Printf("%s not found, would insert at %d", target, i)
//	}
</output>
</example>

Note, the inner part of <output> is only the comment of the documentation WITHOUT the function itself.
ONLY generate the documentation text, nothing else. Do not repeat the input. Do not wrap it in XML tags but keep it as a comment, ie. each new line starts with "//".
Update and correct the pre-existing documentation, parametern names or types might have changed. Wrap everything to 88 chars.
</instruction>

Here is the existing documentation:
<previous_output> ]=====], doc_string, [=====[
</previous_output>
Here is the fucntion that should be documented:
<task> ]=====], hint, [=====[
</task>
]=====] })
    end
  end,

  placement_range = function(lsp_range)
    local start_node = active_doc:node_from_range(lsp_range)
    if start_node == nil then
      return nil
    end
    local fn_node = findup(start_node, allowed_kinds)
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
