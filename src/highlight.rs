use crate::ansi;
use crate::language::Language;

pub fn highlight_line(line: &str, language: Language) -> String {
    if language == Language::PlainText {
        return line.to_string();
    }

    let mut output = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if starts_with(&chars, index, "//") {
            output.push_str(&ansi::paint(
                &chars[index..].iter().collect::<String>(),
                ansi::BRIGHT_BLACK,
            ));
            break;
        }

        if starts_with(&chars, index, "/*") {
            let end = find_after(&chars, index + 2, "*/").unwrap_or(chars.len());
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
            output.push_str(&ansi::paint(
                &chars[index..end].iter().collect::<String>(),
                ansi::GREEN,
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
    use super::highlight_line;
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
    fn leaves_plain_text_unchanged() {
        assert_eq!(
            highlight_line("hello 123", Language::PlainText),
            "hello 123"
        );
    }
}
