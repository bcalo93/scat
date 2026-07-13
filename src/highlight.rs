use crate::ansi;
use crate::language::Language;

#[cfg(test)]
pub fn highlight_line(line: &str, language: Language) -> String {
    let mut highlighter = SimpleHighlighter::new(language);
    highlighter.highlight_line(line)
}

#[cfg(test)]
pub fn highlight_document(content: &str, language: Language) -> String {
    let mut highlighter = SimpleHighlighter::new(language);
    let mut output = String::new();

    for line in content.lines() {
        output.push_str(&highlighter.highlight_line(line));
        output.push('\n');
    }

    output
}

pub trait SyntaxHighlighter {
    fn highlight_line(&mut self, line: &str) -> String;
}

pub struct SimpleHighlighter {
    language: Language,
    in_block_comment: bool,
}

impl SimpleHighlighter {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            in_block_comment: false,
        }
    }
}

impl SyntaxHighlighter for SimpleHighlighter {
    fn highlight_line(&mut self, line: &str) -> String {
        highlight_line_with_state(line, self)
    }
}

fn highlight_line_with_state(line: &str, state: &mut SimpleHighlighter) -> String {
    let language = state.language;
    if language == Language::PlainText {
        return line.to_string();
    }

    let mut output = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if state.in_block_comment {
            let comment_end = find_after(&chars, index, "*/");
            let end = comment_end.unwrap_or(chars.len());
            output.push_str(&ansi::paint(
                &chars[index..end].iter().collect::<String>(),
                ansi::BRIGHT_BLACK,
            ));
            state.in_block_comment = comment_end.is_none();
            index = end;
            continue;
        }

        if starts_with(&chars, index, "//") {
            output.push_str(&ansi::paint(
                &chars[index..].iter().collect::<String>(),
                ansi::BRIGHT_BLACK,
            ));
            break;
        }

        if starts_with(&chars, index, "/*") {
            let comment_end = find_after(&chars, index + 2, "*/");
            let end = comment_end.unwrap_or(chars.len());
            state.in_block_comment = comment_end.is_none();
            output.push_str(&ansi::paint(
                &chars[index..end].iter().collect::<String>(),
                ansi::BRIGHT_BLACK,
            ));
            index = end;
            continue;
        }

        let ch = chars[index];
        if ch == '"' || ch == '\'' || ch == '`' {
            let end = scan_string(&chars, index, ch);
            let token = chars[index..end].iter().collect::<String>();
            let color = if language == Language::Json && next_non_ws_is(&chars, end, ':') {
                ansi::BLUE
            } else {
                ansi::GREEN
            };
            output.push_str(&ansi::paint(&token, color));
            index = end;
            continue;
        }

        if is_jsx(language) && ch == '<' && looks_like_jsx_tag(&chars, index) {
            let end = scan_jsx_tag(&chars, index);
            output.push_str(&highlight_jsx_tag(
                &chars[index..end].iter().collect::<String>(),
            ));
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

        if is_ident_start(ch) {
            let start = index;
            index += 1;
            while index < chars.len() && is_ident_continue(chars[index]) {
                index += 1;
            }
            let word = chars[start..index].iter().collect::<String>();
            if language.keywords().contains(&word.as_str()) {
                output.push_str(&ansi::paint(&word, ansi::MAGENTA));
            } else {
                output.push_str(&word);
            }
            continue;
        }

        if "{}[]()<>:;,.=+-*/!&|?".contains(ch) {
            output.push_str(&ansi::paint(&ch.to_string(), ansi::CYAN));
        } else {
            output.push(ch);
        }
        index += 1;
    }

    output
}

fn highlight_jsx_tag(tag: &str) -> String {
    let mut output = String::new();
    let chars: Vec<char> = tag.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        if ch == '<' || ch == '>' || ch == '/' || ch == '=' || ch == '{' || ch == '}' {
            output.push_str(&ansi::paint(&ch.to_string(), ansi::CYAN));
            index += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            let end = scan_string(&chars, index, ch);
            output.push_str(&ansi::paint(
                &chars[index..end].iter().collect::<String>(),
                ansi::GREEN,
            ));
            index = end;
            continue;
        }

        if is_ident_start(ch) {
            let start = index;
            index += 1;
            while index < chars.len() && is_ident_continue(chars[index]) {
                index += 1;
            }
            let word = chars[start..index].iter().collect::<String>();
            let color = if previous_non_ws_is(&chars, start, '<')
                || previous_non_ws_is(&chars, start, '/')
            {
                ansi::MAGENTA
            } else {
                ansi::BLUE
            };
            output.push_str(&ansi::paint(&word, color));
            continue;
        }

        output.push(ch);
        index += 1;
    }

    output
}

fn starts_with(chars: &[char], index: usize, needle: &str) -> bool {
    needle
        .chars()
        .enumerate()
        .all(|(offset, expected)| chars.get(index + offset) == Some(&expected))
}

fn find_after(chars: &[char], start: usize, needle: &str) -> Option<usize> {
    (start..chars.len()).find_map(|index| {
        if starts_with(chars, index, needle) {
            Some(index + needle.chars().count())
        } else {
            None
        }
    })
}

fn next_non_ws_is(chars: &[char], start: usize, expected: char) -> bool {
    chars
        .iter()
        .skip(start)
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| *ch == expected)
}

fn previous_non_ws_is(chars: &[char], start: usize, expected: char) -> bool {
    chars
        .iter()
        .take(start)
        .rev()
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| *ch == expected)
}

fn is_jsx(language: Language) -> bool {
    matches!(language, Language::Jsx | Language::Tsx)
}

fn looks_like_jsx_tag(chars: &[char], index: usize) -> bool {
    match chars.get(index + 1) {
        Some('/') => true,
        Some(ch) => ch.is_ascii_alphabetic(),
        None => false,
    }
}

fn scan_jsx_tag(chars: &[char], start: usize) -> usize {
    let mut index = start + 1;
    let mut quote = None;
    let mut escaped = false;

    while index < chars.len() {
        let ch = chars[index];
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
        } else if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '>' {
            return index + 1;
        }
        index += 1;
    }

    chars.len()
}

fn scan_string(chars: &[char], start: usize, quote: char) -> usize {
    let mut index = start + 1;
    let mut escaped = false;

    while index < chars.len() {
        let ch = chars[index];
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == quote {
            return index + 1;
        }
        index += 1;
    }

    chars.len()
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::{highlight_document, highlight_line};
    use crate::ansi;
    use crate::language::Language;

    #[test]
    fn highlights_keywords() {
        let highlighted = highlight_line("fn main() {}", Language::Rust);
        assert!(highlighted.contains(&format!("{}fn{}", ansi::MAGENTA, ansi::RESET)));
    }

    #[test]
    fn highlights_strings() {
        let highlighted = highlight_line("const x = \"hello\";", Language::JavaScript);
        assert!(highlighted.contains(&format!("{}\"hello\"{}", ansi::GREEN, ansi::RESET)));
    }

    #[test]
    fn highlights_json_keys_differently_from_values() {
        let highlighted = highlight_line("\"name\": \"mybat\"", Language::Json);
        assert!(highlighted.contains(&format!("{}\"name\"{}", ansi::BLUE, ansi::RESET)));
        assert!(highlighted.contains(&format!("{}\"mybat\"{}", ansi::GREEN, ansi::RESET)));
    }

    #[test]
    fn highlights_multiline_block_comments() {
        let highlighted =
            highlight_document("/* hello\nstill comment */\nfn main() {}\n", Language::Rust);
        assert!(highlighted.contains(&format!(
            "{}still comment */{}",
            ansi::BRIGHT_BLACK,
            ansi::RESET
        )));
        assert!(highlighted.contains(&format!("{}fn{}", ansi::MAGENTA, ansi::RESET)));
    }

    #[test]
    fn highlights_jsx_tags_and_props() {
        let highlighted = highlight_line("<Button title=\"Save\" />", Language::Tsx);
        assert!(highlighted.contains(&format!("{}Button{}", ansi::MAGENTA, ansi::RESET)));
        assert!(highlighted.contains(&format!("{}title{}", ansi::BLUE, ansi::RESET)));
    }

    #[test]
    fn leaves_plain_text_unchanged() {
        assert_eq!(
            highlight_line("hello 123", Language::PlainText),
            "hello 123"
        );
    }
}
