local function findup(node, kind)
  while node ~= nil and node:kind() ~= kind do
    node = node:parent()
  end
  return node
end

local function extract_docstring(fn_node)
  local docstring = ""
  local node = fn_node:prev()
  while node ~= nil and node:kind() == "line_comment" do
    docstring = doc:get_text(node) .. docstring
    node = node:prev()
  end

  return docstring
end

-- necessary functions

function is_triggered(selection_range)
  local cursor_node = doc:get_node(selection_range)
  if cursor_node == nil then
    return false
  end
  local fn_node = findup(cursor_node, "function_item")
  if fn_node ~= nil then
    return true
  end
  return false
end

function action_name()
  return "Update docstring"
end

function process_answer(text, selection_range)
  return text .. "\n"
end

function create_prompt(selection_range)
  local cursor_node = doc:get_node(selection_range)
  if cursor_node == nil then
    return nil
  end
  local fn_node = findup(cursor_node, "function_item")
  if fn_node ~= nil then
    local hint = doc:get_text(fn_node)

    local doc_string = extract_docstring(fn_node)

    return table.concat({
      [=====[
Human: Write a rust docstring for a given function. Follow the style of

  /// Traverse up the AST nodes until a node of the specified type is found.
  ///
  /// # Arguments
  ///
  /// * `node` - The starting node for the search.
  /// * `type_name` - The type of the node to search for.
  ///
  /// # Returns
  ///
  /// An `Option<Node>` containing the found node or `None` if no such node is found.
  pub fn findup<'a>(
      mut node: Option<tree_sitter::Node<'a>>,
      type_name: &str,
  ) -> Option<tree_sitter::Node<'a>> {
      while node.is_some() {
          let inner_node = node.unwrap();
          if inner_node.kind() == type_name {
              return Some(inner_node);
          }
          node = inner_node.parent();
      }
      None
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
end

function placement_range(selection_range)
  local start_node = doc:get_node(selection_range)
  if start_node == nil then
    return nil
  end
  local fn_node = findup(start_node, "function_item")
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
    local p_node = fn_node:prev()
    while p_node ~= nil and p_node:kind() == "line_comment" do
      local range = p_node:range()
      ret.start_line = range.start_line
      p_node = p_node:prev()
    end
    return ret
  end
end
