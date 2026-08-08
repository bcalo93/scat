use super::SyntaxHighlighter;

pub(super) struct PlainTextHighlighter;

impl SyntaxHighlighter for PlainTextHighlighter {
    fn highlight_line(&mut self, line: &str) -> String {
        line.to_string()
    }
}
