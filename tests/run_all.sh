#!/bin/bash

TEST_DIR="tests/cases"

find "$TEST_DIR" -type f -name "*.py" ! -name "*.want.py" | while IFS= read -r file; do
  echo "---"
  echo $file
    if [ -d "$file" ] || [[ "$file" == "." ]] || [[ "$file" == ".." ]]; then
        continue
    fi
    base_filename=$(basename "$file" .py)

    IFS='.' read -r case_name cursor_line cursor_pos ext <<< "$base_filename"
    want_file="${file}.want"
    got_file="${file}.out"
    action_name=$(basename "$(dirname "$file")")

    nvim  -c "edit $file | call cursor($cursor_line, $cursor_pos)" -c "lua test_lsp_code_action(\"$action_name\")" -c 'sleep 4' -c "wq! $got_file" --headless
    # NVIM_APPNAME=nvim-test nvim  -c "edit $file | call cursor($cursor_line, $cursor_pos)" -c "lua test_lsp_code_action(\"$action_name\")" -c 'sleep 4' -c "wq! $got_file" --headless

    if diff -u "$want_file" "$got_file"; then
        echo "Test passed: $file"
    else
        echo "Test failed: $file"
        exit 1
    fi
done

