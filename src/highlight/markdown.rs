use crate::ansi;
use crate::language::Language;

use super::helpers::{find_single_char_after, highlight_jsx_tag, looks_like_jsx_tag, scan_jsx_tag};
use super::jsx::JsxHighlighter;
use super::SyntaxHighlighter;

pub(super) trait MarkdownHighlight: SyntaxHighlighter {
    fn in_code_fence(&self) -> bool;
    fn toggle_code_fence(&mut self);
    fn highlight_inline(&self, line: &str) -> String;

    fn highlight_line_impl(&mut self, line: &str) -> String {
        let trimmed_start = line.trim_start();

        if is_markdown_fence(trimmed_start) {
            self.toggle_code_fence();
            return ansi::paint(line, ansi::BRIGHT_BLACK);
        }

        if self.in_code_fence() {
            return ansi::paint(line, ansi::GREEN);
        }

        if let Some(marker_len) = markdown_heading_marker_len(trimmed_start) {
            let indent_len = line.len() - trimmed_start.len();
            return format!(
                "{}{}{}",
                &line[..indent_len],
                ansi::paint(&trimmed_start[..marker_len], ansi::MAGENTA),
                ansi::paint(&trimmed_start[marker_len..], ansi::BLUE)
            );
        }

        if trimmed_start.starts_with('>') {
            return highlight_markdown_prefixed_line(line, trimmed_start, 1, ansi::BRIGHT_BLACK, self);
        }

        if let Some(marker_len) = markdown_list_marker_len(trimmed_start) {
            return highlight_markdown_prefixed_line(line, trimmed_start, marker_len, ansi::CYAN, self);
        }

        self.highlight_inline(line)
    }
}

pub(super) struct MarkdownHighlighter {
    in_code_fence: bool,
}

impl MarkdownHighlighter {
    pub(super) fn new() -> Self {
        Self {
            in_code_fence: false,
        }
    }
}

impl MarkdownHighlight for MarkdownHighlighter {
    fn in_code_fence(&self) -> bool {
        self.in_code_fence
    }

    fn toggle_code_fence(&mut self) {
        self.in_code_fence = !self.in_code_fence;
    }

    fn highlight_inline(&self, line: &str) -> String {
        highlight_markdown_inline_pure(line)
    }
}

impl SyntaxHighlighter for MarkdownHighlighter {
    fn highlight_line(&mut self, line: &str) -> String {
        self.highlight_line_impl(line)
    }
}

pub(super) struct MdxHighlighter {
    in_code_fence: bool,
}

impl MdxHighlighter {
    pub(super) fn new() -> Self {
        Self {
            in_code_fence: false,
        }
    }
}

impl MarkdownHighlight for MdxHighlighter {
    fn in_code_fence(&self) -> bool {
        self.in_code_fence
    }

    fn toggle_code_fence(&mut self) {
        self.in_code_fence = !self.in_code_fence;
    }

    fn highlight_inline(&self, line: &str) -> String {
        highlight_markdown_inline_with_jsx(line)
    }
}

impl SyntaxHighlighter for MdxHighlighter {
    fn highlight_line(&mut self, line: &str) -> String {
        let trimmed_start = line.trim_start();

        if looks_like_mdx_code_line(trimmed_start) {
            let mut highlighter = JsxHighlighter::new(Language::Tsx);
            return highlighter.highlight_line(line);
        }

        self.highlight_line_impl(line)
    }
}

fn looks_like_mdx_code_line(trimmed_start: &str) -> bool {
    trimmed_start.starts_with("import ")
        || trimmed_start.starts_with("export ")
        || trimmed_start.starts_with('<')
}

fn is_markdown_fence(trimmed_start: &str) -> bool {
    trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~")
}

fn markdown_heading_marker_len(trimmed_start: &str) -> Option<usize> {
    let marker_len = trimmed_start.chars().take_while(|ch| *ch == '#').count();
    if (1..=6).contains(&marker_len)
        && trimmed_start
            .chars()
            .nth(marker_len)
            .is_some_and(|ch| ch.is_whitespace())
    {
        Some(marker_len)
    } else {
        None
    }
}

fn markdown_list_marker_len(trimmed_start: &str) -> Option<usize> {
    let chars: Vec<char> = trimmed_start.chars().collect();
    match chars.as_slice() {
        ['-', next, ..] | ['*', next, ..] | ['+', next, ..] if next.is_whitespace() => Some(1),
        _ => {
            let digits = chars.iter().take_while(|ch| ch.is_ascii_digit()).count();
            if digits > 0
                && chars.get(digits) == Some(&'.')
                && chars.get(digits + 1).is_some_and(|ch| ch.is_whitespace())
            {
                Some(digits + 1)
            } else {
                None
            }
        }
    }
}

fn highlight_markdown_prefixed_line<M: MarkdownHighlight + ?Sized>(
    line: &str,
    trimmed_start: &str,
    marker_len: usize,
    color: &str,
    highlighter: &M,
) -> String {
    let indent_len = line.len() - trimmed_start.len();
    format!(
        "{}{}{}",
        &line[..indent_len],
        ansi::paint(&trimmed_start[..marker_len], color),
        highlighter.highlight_inline(&trimmed_start[marker_len..])
    )
}

fn highlight_markdown_inline_pure(line: &str) -> String {
    let mut output = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '`' {
            let end = find_single_char_after(&chars, index + 1, '`').unwrap_or(chars.len() - 1);
            output.push_str(&ansi::paint(
                &chars[index..=end].iter().collect::<String>(),
                ansi::GREEN,
            ));
            index = end + 1;
            continue;
        }

        if chars[index] == '[' {
            if let Some(end) = scan_markdown_link(&chars, index) {
                output.push_str(&highlight_markdown_link(
                    &chars[index..end].iter().collect::<String>(),
                ));
                index = end;
                continue;
            }
        }

        if (chars[index] == '*' || chars[index] == '_')
            && chars
                .get(index + 1)
                .is_some_and(|next| !next.is_whitespace())
        {
            output.push_str(&ansi::paint(&chars[index].to_string(), ansi::MAGENTA));
            index += 1;
            continue;
        }

        output.push(chars[index]);
        index += 1;
    }

    output
}

fn highlight_markdown_inline_with_jsx(line: &str) -> String {
    let mut output = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '`' {
            let end = find_single_char_after(&chars, index + 1, '`').unwrap_or(chars.len() - 1);
            output.push_str(&ansi::paint(
                &chars[index..=end].iter().collect::<String>(),
                ansi::GREEN,
            ));
            index = end + 1;
            continue;
        }

        if chars[index] == '[' {
            if let Some(end) = scan_markdown_link(&chars, index) {
                output.push_str(&highlight_markdown_link(
                    &chars[index..end].iter().collect::<String>(),
                ));
                index = end;
                continue;
            }
        }

        if chars[index] == '<' && looks_like_jsx_tag(&chars, index) {
            let end = scan_jsx_tag(&chars, index);
            output.push_str(&highlight_jsx_tag(
                &chars[index..end].iter().collect::<String>(),
            ));
            index = end;
            continue;
        }

        if (chars[index] == '*' || chars[index] == '_')
            && chars
                .get(index + 1)
                .is_some_and(|next| !next.is_whitespace())
        {
            output.push_str(&ansi::paint(&chars[index].to_string(), ansi::MAGENTA));
            index += 1;
            continue;
        }

        output.push(chars[index]);
        index += 1;
    }

    output
}

fn scan_markdown_link(chars: &[char], start: usize) -> Option<usize> {
    let close_bracket = find_single_char_after(chars, start + 1, ']')?;
    if chars.get(close_bracket + 1) != Some(&'(') {
        return None;
    }
    let close_paren = find_single_char_after(chars, close_bracket + 2, ')')?;
    Some(close_paren + 1)
}

fn highlight_markdown_link(link: &str) -> String {
    let mut output = String::new();
    let chars: Vec<char> = link.chars().collect();
    let close_bracket = chars
        .iter()
        .position(|ch| *ch == ']')
        .expect("link has a closing bracket");

    output.push_str(&ansi::paint("[", ansi::CYAN));
    output.push_str(&ansi::paint(
        &chars[1..close_bracket].iter().collect::<String>(),
        ansi::BLUE,
    ));
    output.push_str(&ansi::paint("](", ansi::CYAN));
    output.push_str(&ansi::paint(
        &chars[close_bracket + 2..chars.len() - 1]
            .iter()
            .collect::<String>(),
        ansi::GREEN,
    ));
    output.push_str(&ansi::paint(")", ansi::CYAN));
    output
}
