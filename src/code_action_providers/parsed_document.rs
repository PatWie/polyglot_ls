use tower_lsp::lsp_types::Url;
use tree_sitter::{Node, Parser, Point, Query, QueryCursor, Tree};

pub struct ParsedDocument {
    pub tree: Tree,
    parser: Parser,
    source: String,
    // TODO(patwie): Maybe we go full UTF16 ranges and use IndexedText<String> here.
    language: String,
    pub uri: Url,
}

fn create_parser(lang: &str) -> Parser {
    let mut parser = Parser::new();
    let language = match lang {
        "python" => tree_sitter_python::language(),
        "rust" => tree_sitter_rust::language(),
        "go" => tree_sitter_go::language(),
        // TODO(patwie): Better error handling
        _ => tree_sitter_python::language(),
    };

    parser
        .set_language(&language)
        .expect("Error loading Python grammar");
    parser
}

fn get_subtext(
    text: &str,
    start_line: usize,
    start_char: usize,
    end_line: usize,
    end_char: usize,
) -> String {
    let mut lines: Vec<&str> = text.split('\n').collect();

    if start_line >= lines.len() || end_line >= lines.len() {
        return "".to_owned();
    }

    let mut start_text = lines[start_line];
    let mut end_text = lines[end_line];

    if start_line == end_line {
        start_text[start_char..end_char].to_owned()
    } else {
        start_text = &start_text[start_char..];
        end_text = &end_text[..end_char];

        let mut result = String::new();
        result.push_str(start_text);
        for line in &mut lines[start_line + 1..end_line] {
            result.push('\n');
            result.push_str(line);
        }
        result.push('\n');
        result.push_str(end_text);
        result
    }
}

impl ParsedDocument {
    pub fn new(source: &str, uri: &Url, language: &str) -> Self {
        let mut parser = create_parser(language);
        let tree = parser.parse(source, None).unwrap();
        Self {
            tree,
            parser,
            source: source.to_string(),
            language: language.to_string(),
            uri: uri.to_owned(),
        }
    }
    pub fn duplicate(&self) -> Self {
        let mut parser = create_parser(&self.language);
        let tree = parser.parse(self.source.clone(), None).unwrap();
        Self {
            tree,
            parser,
            language: self.language.clone(),
            source: self.source.to_string(),
            uri: self.uri.to_owned(),
        }
    }

    pub fn update(&mut self, source: &str) {
        self.tree = self.parser.parse(source, Some(&self.tree)).unwrap();
        self.source = source.to_string();
    }

    pub fn get_ts_node_for_range(&self, range: &tower_lsp::lsp_types::Range) -> Option<Node> {
        let start = Point::new(range.start.line as usize, range.start.character as usize);
        let end = Point::new(range.end.line as usize, range.end.character as usize);
        self.tree.root_node().descendant_for_point_range(start, end)
    }

    pub fn text_from_node(&self, node: &Node) -> String {
        node.utf8_text(self.source.as_bytes())
            .expect("can find text")
            .to_string()
    }

    pub fn text_from_range(&self, range: &tower_lsp::lsp_types::Range) -> String {
        get_subtext(
            &self.source,
            range.start.line as usize,
            range.start.character as usize,
            range.end.line as usize,
            range.end.character as usize,
        )
    }
    pub fn query<'a>(&'a self, node: &'a Node, query: &str) -> Vec<Node> {
        let q = Query::new(&self.tree.language(), query);
        if q.is_err() {
            return Vec::default();
        }
        let q = q.unwrap();
        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(node.byte_range());

        let matches = cursor.matches(&q, *node, self.source.as_bytes());
        let mut nodes = Vec::new();

        for m in matches {
            for capture in m.captures {
                nodes.push(capture.node);
            }
        }
        nodes
    }

    pub fn find_first<'a>(&'a self, node: &'a Node, query: &str) -> Option<Node> {
        let q = Query::new(&self.tree.language(), query).unwrap();
        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(node.byte_range());
        let first_match = cursor
            .matches(&q, *node, self.source.as_bytes())
            .flat_map(|m| m.captures)
            .next();
        first_match.map(|m| m.node)
    }
}
