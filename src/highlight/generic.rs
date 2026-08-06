use crate::ansi;
use crate::language::Language;

use super::helpers::{
    find_after, highlight_jsx_tag, is_ident_continue, is_ident_start, is_jsx, looks_like_jsx_tag,
    paint_maybe, scan_jsx_tag, scan_string, starts_with,
};
use super::SyntaxHighlighter;

pub struct GenericHighlighter {
    pub language: Language,
    pub in_block_comment: bool,
}

impl GenericHighlighter {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            in_block_comment: false,
        }
    }

    pub fn highlight_token(&mut self, line: &str, index: usize) -> Option<(String, usize)> {
        let chars: Vec<char> = line.chars().collect();
        if index >= chars.len() {
            return None;
        }

        if self.in_block_comment {
            let comment_end = find_after(&chars, index, "*/");
            let end = comment_end.unwrap_or(chars.len());
            self.in_block_comment = comment_end.is_none();
            return Some((
                ansi::paint(&chars[index..end].iter().collect::<String>(), ansi::BRIGHT_BLACK),
                end,
            ));
        }

        if starts_with(&chars, index, "//") {
            return Some((
                ansi::paint(&chars[index..].iter().collect::<String>(), ansi::BRIGHT_BLACK),
                chars.len(),
            ));
        }

        if starts_with(&chars, index, "/*") {
            let comment_end = find_after(&chars, index + 2, "*/");
            let end = comment_end.unwrap_or(chars.len());
            self.in_block_comment = comment_end.is_none();
            return Some((
                ansi::paint(&chars[index..end].iter().collect::<String>(), ansi::BRIGHT_BLACK),
                end,
            ));
        }

        let ch = chars[index];

        if ch == '"' || ch == '\'' || ch == '`' {
            let end = scan_string(&chars, index, ch);
            return Some((
                ansi::paint(&chars[index..end].iter().collect::<String>(), ansi::GREEN),
                end,
            ));
        }

        if is_jsx(self.language) && ch == '<' && looks_like_jsx_tag(&chars, index) {
            let end = scan_jsx_tag(&chars, index);
            return Some((
                highlight_jsx_tag(&chars[index..end].iter().collect::<String>()),
                end,
            ));
        }

        if ch.is_ascii_digit() {
            let start = index;
            let mut i = index + 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            return Some((
                ansi::paint(&chars[start..i].iter().collect::<String>(), ansi::YELLOW),
                i,
            ));
        }

        if is_ident_start(ch) {
            let start = index;
            let mut i = index + 1;
            while i < chars.len() && is_ident_continue(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let color = if self.language.keywords().contains(&word.as_str()) {
                ansi::MAGENTA
            } else {
                ""
            };
            return Some((paint_maybe(&word, color), i));
        }

        if "{}[]()<>:;,.=+-*/!&|?".contains(ch) {
            return Some((ansi::paint(&ch.to_string(), ansi::CYAN), index + 1));
        }

        Some((ch.to_string(), index + 1))
    }
}

impl SyntaxHighlighter for GenericHighlighter {
    fn highlight_line(&mut self, line: &str) -> String {
        let mut output = String::new();
        let mut index = 0;
        let len = line.chars().count();

        while index < len {
            match self.highlight_token(line, index) {
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
