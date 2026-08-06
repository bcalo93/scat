use crate::language::Language;

use super::generic::GenericHighlighter;
use super::helpers::{highlight_jsx_tag, looks_like_jsx_tag, scan_jsx_tag};
use super::SyntaxHighlighter;

pub(super) struct JsxHighlighter {
    inner: GenericHighlighter,
}

impl JsxHighlighter {
    pub(super) fn new(language: Language) -> Self {
        Self {
            inner: GenericHighlighter::new(language),
        }
    }
}

impl SyntaxHighlighter for JsxHighlighter {
    fn highlight_line(&mut self, line: &str) -> String {
        let chars: Vec<char> = line.chars().collect();
        let mut output = String::new();
        let mut index = 0;

        while index < chars.len() {
            let ch = chars[index];

            if !self.inner.in_block_comment
                && ch == '<'
                && looks_like_jsx_tag(&chars, index)
            {
                let end = scan_jsx_tag(&chars, index);
                output.push_str(&highlight_jsx_tag(
                    &chars[index..end].iter().collect::<String>(),
                ));
                index = end;
                continue;
            }

            match self.inner.highlight_token(line, index) {
                Some((highlighted, next_index)) => {
                    output.push_str(&highlighted);
                    index = next_index;
                }
                None => break,
            }
        }

        output
    }
}
