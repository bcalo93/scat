use crate::ansi;
use crate::language::Language;

use super::generic::GenericHighlighter;
use super::helpers::{
    find_single_char_after, highlight_jsx_tag, looks_like_jsx_tag, scan_jsx_tag,
};
use super::SyntaxHighlighter;

pub(super) struct MarkdownHighlighter {
    language: Language,
    in_markdown_code_fence: bool,
}

impl MarkdownHighlighter {
    pub(super) fn new(language: Language) -> Self {
        Self {
            language,
            in_markdown_code_fence: false,
        }
    }
}

impl SyntaxHighlighter for MarkdownHighlighter {
    fn highlight_line(&mut self, line: &str) -> String {
        let trimmed_start = line.trim_start();
        let is_mdx = self.language == Language::Mdx;

        if is_markdown_fence(trimmed_start) {
            self.in_markdown_code_fence = !self.in_markdown_code_fence;
            return ansi::paint(line, ansi::BRIGHT_BLACK);
        }

        if self.in_markdown_code_fence {
            return ansi::paint(line, ansi::GREEN);
        }

        if is_mdx && looks_like_mdx_code_line(trimmed_start) {
            let mut highlighter = GenericHighlighter::new(Language::Tsx);
            return highlighter.highlight_line(line);
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
            return highlight_markdown_prefixed_line(
                line,
                trimmed_start,
                1,
                ansi::BRIGHT_BLACK,
                is_mdx,
            );
        }

        if let Some(marker_len) = markdown_list_marker_len(trimmed_start) {
            return highlight_markdown_prefixed_line(
                line,
                trimmed_start,
                marker_len,
                ansi::CYAN,
                is_mdx,
            );
        }

        highlight_markdown_inline(line, is_mdx)
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

fn highlight_markdown_prefixed_line(
    line: &str,
    trimmed_start: &str,
    marker_len: usize,
    color: &str,
    is_mdx: bool,
) -> String {
    let indent_len = line.len() - trimmed_start.len();
    format!(
        "{}{}{}",
        &line[..indent_len],
        ansi::paint(&trimmed_start[..marker_len], color),
        highlight_markdown_inline(&trimmed_start[marker_len..], is_mdx)
    )
}

fn highlight_markdown_inline(line: &str, is_mdx: bool) -> String {
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

        if is_mdx && chars[index] == '<' && looks_like_jsx_tag(&chars, index) {
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
