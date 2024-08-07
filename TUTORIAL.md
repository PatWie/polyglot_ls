# Configuration Tutorial

## Lua

Each behavior is implemented in Lua with the function

```lua
local M = {
-- range -> bool
-- can code-action be used in the current selection?
  is_triggered = function(selected_code_range)
    return true
  end,

-- void -> string
-- name of code-action
  action_name = function()
    return "Improve Wording"
  end,

-- string, range -> string
-- post-process any LLM answer
  process_answer = function(llm_answer, selected_code_range)
    return llm_answer
  end,

-- range -> string
-- create a prompt string for the llm
  create_prompt = function(selected_code_range)
    local select_text = active_doc:text_from_range(selected_code_range)
    return table.concat({
      [=====[ Human:
      Improve the wording, fix grammar and typos. DO NOT output anything else. Just the improved text. No explanation.
]=====], select_text, [=====[
Assistant: ]=====] })
  end,

-- range -> range
-- where to place the post-processed llm-answer
  placement_range = function(selected_code_range)
    return selected_code_range
  end
}

return M
```

Best is to inspect some existing code-actions, e.g., those [code actions for
rust](./config/code_actions/rust).

## YAML

Behavior can be configured in YAML. This approach may be simpler, but it also
has limitations in terms of functionality. I recommend using Lua instead. Let's
start by adding a function to add a docstring to a Python function.

```python
def add(a, b):
    return a + b
```

We want the output:

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

To add such functionality, we first need to specify the trigger. Open the
[Tree-sitter Playground](https://tree-sitter.github.io/tree-sitter/playground)
and place the code above. According to the playground, this code action should
be enabled whenever the cursor is within a `function_definition` node.

```yaml
triggers:
  - kind: function_definition
    relation: findup
```

This will start in the current node under the cursor and traverse the AST up
until the first `function_definition` node. If no such node is found, the
action will be disabled.

To form a prompt, we also want to inform the LLM about some _context_ for the
action. In this case, we use the entire function (we could also use the entire
source code via `kind=module`).

```yaml
context:
  kind: function_definition
  relation: findup # findup | exact
  hints:
    - name: FUNCTION_CONTEXT
      query: ((function_definition) @function)
```

This is the start node to extract hints, like parameters, function body, and so
on, which can be interpolated into the prompt. In our case, we use the entire
function as a query. Those query results will interpolated into the the
prompt-template as a hint:

```yaml
prompt_template: |
  Do this or that for <<<FUNCTION_CONTEXT>>>
```

The answer from the LLM might need some post-processing (e.g., adding
brackets), which can be configured via an optional `answer_template`:

```yaml
answer_template: "{<<ANSWER>>>}"
```

The last piece is to tell the front-end where the answer should be placed. This
is a list of possible tree-sitter queries, which are walked in order, and
the first captured match will determine the target range. For Python functions,
we first want to try replacing the existing docstring. If this is not possible,
we will add a new docstring (before the function body).

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

See the `config/code_actions/` directory for more examples.
