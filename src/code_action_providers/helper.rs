use tower_lsp::lsp_types::{Position, Range};
use tree_sitter::Point;

// Convert Tree-sitter Point to LSP Position
pub fn ts_point_to_lsp_position(point: &Point) -> Position {
    Position {
        line: point.row as u32,
        character: point.column as u32,
    }
}

// Convert LSP Position to Tree-sitter Point
pub fn lsp_position_to_ts_point(position: &Position) -> Point {
    Point {
        row: position.line as usize,
        column: position.character as usize,
    }
}

// Convert Tree-sitter Range to LSP Range
pub fn ts_node_to_lsp_range(node: &tree_sitter::Node) -> Range {
    let start = ts_point_to_lsp_position(&node.start_position());
    let end = ts_point_to_lsp_position(&node.end_position());
    Range { start, end }
}
pub fn prepend_ts_node_to_lsp_range(node: &tree_sitter::Node) -> Range {
    let mut pos = ts_point_to_lsp_position(&node.start_position());
    pos.character = 0;
    Range {
        start: pos,
        end: pos,
    }
}

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

pub fn indent_text(text: &str, indent_amount: usize) -> String {
    let indent = " ".repeat(indent_amount);
    trim_last_newline(
        &text
            .lines()
            .map(|line| format!("{}{}\n", indent, line))
            .collect::<String>(),
    )
}

fn trim_last_newline(input: &str) -> String {
    let mut result = input.to_string();
    if result.ends_with('\n') {
        result.pop();
    }
    result
}
