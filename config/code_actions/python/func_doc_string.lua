local function findup(node, kind)
  while node ~= nil and node:kind() ~= kind do
    node = node:parent()
  end
  return node
end



local function find_specific_node(start_range, kind)
  local start_node = doc:get_node(start_range)
  if start_node == nil then
    return nil
  end
  return findup(start_node, kind)
end

local function find_docstring(fn_node)
  return doc:query_first(fn_node, [[(function_definition
            body: (block
              (expression_statement
                (string) @docstring)))]])
end

local function find_body(fn_node)
  return doc:query_first(fn_node, [[(function_definition
            body: (block) @body)]])
end

-- required

function is_triggered(selection_range)
  return find_specific_node(selection_range, "function_definition") ~= nil
end

function action_name()
  return "Update Function Docstring"
end

function process_answer(text, selection_range)
  local fn_node = find_specific_node(selection_range, "function_definition")

  local doc_node = find_docstring(fn_node)
  if doc_node ~= nil then
    -- Indent with docstring indentation
    local r = doc_node:range()
    return helper.trim_suffix(helper.indent_text(text, r.start_character), "\n")
  end

  -- Indent with body node indentation
  local body_node = find_body(fn_node)
  local r = body_node:range()
  -- As we insert a new content, we need to add a newline.
  return helper.indent_text(text, r.start_character)
end

function create_prompt(selection_range)
  local fn_node = find_specific_node(selection_range, "function_definition")
  if fn_node ~= nil then
    local function_text = doc:get_text(fn_node)
    print(function_text)


    return table.concat({
      [=====[ Human:
      Write a google style docstring for a given function. Here is an example
      for

        def fetch_smalltable_rows(
            table_handle: smalltable.Table,
            keys: Sequence[bytes | str],
            require_all_keys: bool = False,
        ) -> Mapping[bytes, tuple[str, ...]]:

      how it can look like

        """Fetch rows from a Smalltable.

        Retrieves rows pertaining to the given keys from the Table instance
        represented by table_handle.  String keys will be UTF-8 encoded.

        Args:
            table_handle: An open smalltable.Table instance.
            keys: A sequence of strings representing the key of each table
              row to fetch.  String keys will be UTF-8 encoded.
            require_all_keys: If True only rows with values set for all keys will be
              returned.

        Returns:
            A dict mapping keys to the corresponding table row data
            fetched. Each row is represented as a tuple of strings. For
            example:

            {b'Serak': ('Rigel VII', 'Preparer'),
             b'Zim': ('Irk', 'Invader'),
             b'Lrrr': ('Omicron Persei 8', 'Emperor')}

            Returned keys are always bytes.  If a key from the keys argument is
            missing from the dictionary, then that row was not found in the
            table (and require_all_keys must have been False).

        Raises:
            IOError: An error occurred accessing the smalltable.

        Examples:
            >>> my_table = fetch_smalltable_rows(handle, ["id", "user"], True)
        """

      NEVER write anything else besides the docstring block. ONLY generate the docstring,
      It should include Args, Returns, Raise, Yield, Attributes, Notes, Example if necessary. First line must be in imperative mood. Do NOT output anything else after the docstring.
      Update and correct the pre-existing docstring, parametern names or types might have changed. Wrap everything to 88 chars.
      NEVER write back the initial code, JUST the docstring itself.

      Here is the task:
<task> ]=====], function_text, [=====[
</task>
Assistant: ]=====] })
  end
end

function placement_range(selection_range)
  local fn_node = find_specific_node(selection_range, "function_definition")
  if fn_node == nil then
    return nil
  end

  local doc_node = find_docstring(fn_node)
  if doc_node ~= nil then
    -- Place instead
    local r = doc_node:range()
    r.start_character = 0
    return r
  end

  local body_node = find_body(fn_node)
  -- Place before
  local r = body_node:range()
  return {
    start_line = r.start_line,
    start_character = 0,
    end_line = r.start_line,
    end_character = 0,
  }
end
