use tower_lsp::lsp_types::Url;
use tree_sitter::{Node, Parser, Point, Query, QueryCapture, QueryCursor, Tree};

pub struct ParsedDocument {
    tree: Tree,
    parser: Parser,
    source: String,
    pub uri: Url,
}

fn create_parser() -> Parser {
    let mut parser = Parser::new();
    let language = tree_sitter_python::language();
    parser
        .set_language(&language)
        .expect("Error loading Python grammar");
    parser
}

impl ParsedDocument {
    pub fn new(source: &str, uri: &Url) -> Self {
        let mut parser = create_parser();
        let tree = parser.parse(&source, None).unwrap();
        Self {
            tree,
            parser,
            source: source.to_string(),
            uri: uri.to_owned(),
        }
    }

    pub fn update(&mut self, source: &str) {
        self.tree = self.parser.parse(&source, Some(&self.tree)).unwrap();
        self.source = source.to_string();
    }

    pub fn get_ts_node_for_range(&self, range: &tower_lsp::lsp_types::Range) -> Option<Node> {
        let start = Point::new(range.start.line as usize, range.start.character as usize);
        let end = Point::new(range.end.line as usize, range.end.character as usize);
        self.tree.root_node().descendant_for_point_range(start, end)
    }

    pub fn get_text(&self, node: &Node) -> String {
        node.utf8_text(self.source.as_str().as_bytes())
            .expect("can find text")
            .to_string()
    }

    pub fn find_first<'a>(&'a self, node: &'a Node, query: &str) -> Option<Node> {
        let q = Query::new(&self.tree.language(), query).unwrap();
        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(node.byte_range());
        let first_match = cursor
            .matches(&q, *node, self.source.as_bytes())
            .flat_map(|m| m.captures)
            .next();
        if let Some(m) = first_match {
            return Some(m.node.clone());
        } else {
            None
        }
    }
}
