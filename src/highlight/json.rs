use crate::ansi;

use super::helpers::{next_non_ws_is, scan_string};
use super::SyntaxHighlighter;

pub(super) struct JsonHighlighter;

impl SyntaxHighlighter for JsonHighlighter {
    fn highlight_line(&mut self, line: &str) -> String {
        let mut output = String::new();
        let chars: Vec<char> = line.chars().collect();
        let mut index = 0;

        while index < chars.len() {
            let ch = chars[index];

            if ch == '"' || ch == '\'' || ch == '`' {
                let end = scan_string(&chars, index, ch);
                let token: String = chars[index..end].iter().collect();
                let color = if next_non_ws_is(&chars, end, ':') {
                    ansi::BLUE
                } else {
                    ansi::GREEN
                };
                output.push_str(&ansi::paint(&token, color));
                index = end;
                continue;
            }

            if ch.is_ascii_digit() {
                let start = index;
                index += 1;
                while index < chars.len() && (chars[index].is_ascii_digit() || chars[index] == '.') {
                    index += 1;
                }
                output.push_str(&ansi::paint(
                    &chars[start..index].iter().collect::<String>(),
                    ansi::YELLOW,
                ));
                continue;
            }

            if "{}[](),:=+-*/!".contains(ch) {
                output.push_str(&ansi::paint(&ch.to_string(), ansi::CYAN));
            } else {
                output.push(ch);
            }
            index += 1;
        }

        output
    }
}
