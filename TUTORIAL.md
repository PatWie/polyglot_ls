# Configuration Tutorial

## Lua Configuration

Each behavior is implemented in Lua using the following structure:

```lua
local M = {
  --- Determines if the code-action can be used in the current selection.
  -- @param lsp_range Range: The current selection or cursor position from the editor frontend via LSP.
  -- @return boolean: True if the code-action can be used, otherwise false.
  is_triggered = function(lsp_range)
    return true
  end,

  --- Returns the name of the code-action.
  -- @return string: The name of the code-action.
  get_action_name = function()
    return "Improve Wording"
  end,

  --- Post-processes any LLM reponse.
  -- @param llm_response string: The answer from the LLM.
  -- @param lsp_range Range: The current selection or cursor position from the editor frontend via LSP.
  -- @return string: The post-processed LLM answer.
  process_answer = function(llm_response, lsp_range)
    return llm_response
  end,

  --- Creates a prompt string for the LLM.
  -- @param lsp_range Range: The current selection or cursor position from the editor frontend via LSP.
  -- @return string: The prompt string for the LLM.
  create_prompt = function(lsp_range)
    local selected_text = active_doc:text_from_range(lsp_range)
    return table.concat({
      [=====[
      Human:
      Improve the wording, fix grammar and typos. DO NOT output anything else. Just the improved text. No explanation.
]=====], selected_text, [=====[
Assistant: ]=====]
    })
  end,

  --- Determines where to place the post-processed LLM answer.
  -- @param lsp_range Range: The current selection or cursor position from the editor frontend via LSP.
  -- @return Range: The range where the post-processed LLM answer should be placed.
  get_placement_range = function(lsp_range)
    return lsp_range
  end
}

return M
```

It is recommended to inspect some existing code-actions for better
understanding, such as those for [Rust](./config/code_actions/rust).

## YAML Configuration

Behavior can also be configured in YAML. This approach is simpler but offers
limited functionality compared to Lua. Below is an example to add a docstring
to a Python function.

### Example Python Function

```python
def add(a, b):
    return a + b
```

### Desired Output

```python
def add(a, b):
    """Add two numbers.

    Args:
        a (int or float): The first number to add.
        b (int or float): The second number to add.

    Returns:
        int or float: The sum of a and b.

    Examples:
        >>> add(2, 3)
        5
        >>> add(2.5, 3.2)
        5.7
    """
    return a + b
```

### Configuration Steps

1. **Specify the Trigger**:
   Using the [Tree-sitter
   Playground](https://tree-sitter.github.io/tree-sitter/playground), determine
   that this code action should be enabled whenever the cursor is within a
   `function_definition` node.

   ```yaml
   triggers:
     - kind: function_definition
       relation: findup
   ```

   This configuration starts at the current node under the cursor and traverses
   the AST up until the first `function_definition` node is found. If no such
   node is found, the action is disabled.

2. **Form the Prompt**:
   Provide context to the LLM by using the entire function. Context can be more
   comprehensive, such as using the entire source code.

   ```yaml
   context:
     kind: function_definition
     relation: findup # findup | exact
     hints:
       - name: FUNCTION_CONTEXT
         query: ((function_definition) @function)
   ```

   This sets the start node to extract hints (like parameters and function
   body) to be interpolated into the prompt.

3. **Define the Prompt Template**:
   Use the extracted hints in the prompt template.

   ```yaml
   prompt_template: |
     Generate a comprehensive docstring for the following function: <<<FUNCTION_CONTEXT>>>
   ```

4. **Post-process the Answer**:
   Optionally post-process the LLM's answer using an `answer_template`.

   ```yaml
   answer_template: "{<<ANSWER>>>}"
   ```

5. **Determine the Placement**:
   Specify where the processed answer should be placed using tree-sitter
   queries. For Python functions, try replacing the existing docstring first,
   then add a new docstring if none exists.

   ```yaml
   placement_strategies:
     # Try to find the docstring node
     - query: |
         (function_definition
           body: (block
             (expression_statement
               (string) @docstring)))
       position: replace_block
     # If not existent, find the body node and place it before
     - query: |
         (function_definition
           body: (block) @body)
       position: before
   ```

Refer to the `config/code_actions/` directory for more examples.


