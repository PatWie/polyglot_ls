local function findup(node, kind)
  while node ~= nil and node:kind() ~= kind do
    node = node:parent()
  end
  return node
end

local function extract_fn_name(fn_node)
  for i = 0, fn_node:named_child_count() - 1 do
    local child = fn_node:named_child(i)
    if child:kind() == "identifier" then
      return active_doc:text_from_node(child)
    end
  end
  return nil
end

local function find_mod_end(node)
  local query = [[
    (mod_item) @mod
  ]]
  local matches = active_doc:query(node, query)
  local last_child = nil
  for _, match in ipairs(matches) do
    last_child = match.mod
  end
  return last_child
end

local function find_test_module(root_node)
  for i = 0, root_node:named_child_count() - 1 do
    local child = root_node:named_child(i)
    if child:kind() == "mod_item" then
      local previous_sibling = child:prev_sibling()
      while previous_sibling and previous_sibling:kind() == "attribute_item" do
        local attr_text = active_doc:text_from_node(previous_sibling)
        if attr_text:match("#%[cfg%(test%)%]") then
          return child
        end
        previous_sibling = previous_sibling:prev_sibling()
      end
    end
  end
  return nil
end

return {
  is_triggered = function(selection_range)
    local cursor_node = active_doc:node_from_range(selection_range)
    if cursor_node == nil then
      return false
    end
    return findup(cursor_node, "function_item") ~= nil
  end,

  action_name = function()
    return "Add Test Cases"
  end,

  process_answer = function(text, selection_range)
    print("--------")
    print(text)
    print("--------")
    local root_node = active_doc:root()
    if find_test_module(root_node) == nil then
      return "#[cfg(test)]\nmod tests {\n    use super::*;\n\n    " .. text .. "\n}\n"
    else
      return text .. "\n"
    end
  end,

  create_prompt = function(selection_range)
    local cursor_node = active_doc:node_from_range(selection_range)
    if cursor_node == nil then
      return nil
    end
    local fn_node = findup(cursor_node, "function_item")
    if fn_node ~= nil then
      local fn_name = extract_fn_name(fn_node)
      local fn_text = active_doc:text_from_node(fn_node)
      if fn_name ~= nil then
        return [[
        Human:

        Write meaningful test cases for the rust function ']] .. fn_name .. [['".

As an example, for the input rust function

  pub fn add(a: i32, b: i32) -> i32 {
      a + b
  }

The output test cases would be

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }

    #[test]
    fn test_bad_add() {
        assert_neq!(bad_add(1, 3), 3);
    }



NEVER write anything else besides the test-cases. Do NOT output any explanation. Just output the code.
The output should be only rust code. No extra text!


Here is the task:
<task> ]] .. fn_text .. [[
</task>
        ]]
      end
    end
  end,

  placement_range = function(selection_range)
    local root_node = active_doc:root()
    if root_node == nil then
      return nil
    end
    local test_mod_node = find_test_module(root_node)
    if test_mod_node ~= nil then
      local mod_body = test_mod_node:child_by_field_name("body")
      if mod_body ~= nil then
        local mod_end_range = mod_body:range()
        return {
          start_line = mod_end_range.end_line,
          start_character = 0,
          end_line = mod_end_range.end_line,
          end_character = 0,
        }
      end
    else
      local last_mod_or_fn_node = find_mod_end(root_node)
      if last_mod_or_fn_node ~= nil then
        local mod_end_range = last_mod_or_fn_node:range()
        return {
          start_line = mod_end_range.end_line + 1,
          start_character = 0,
          end_line = mod_end_range.end_line + 1,
          end_character = 0,
        }
      else
        local file_end_line = active_doc:root():range().end_line
        return {
          start_line = file_end_line,
          start_character = 0,
          end_line = file_end_line,
          end_character = 0,
        }
      end
    end
  end
}
