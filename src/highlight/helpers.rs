use crate::ansi;

pub(crate) fn scan_string(chars: &[char], start: usize, quote: char) -> usize {
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

pub(crate) fn starts_with(chars: &[char], index: usize, needle: &str) -> bool {
    needle
        .chars()
        .enumerate()
        .all(|(offset, expected)| chars.get(index + offset) == Some(&expected))
}

pub(crate) fn find_after(chars: &[char], start: usize, needle: &str) -> Option<usize> {
    (start..chars.len()).find_map(|index| {
        if starts_with(chars, index, needle) {
            Some(index + needle.chars().count())
        } else {
            None
        }
    })
}

pub(crate) fn next_non_ws_is(chars: &[char], start: usize, expected: char) -> bool {
    chars
        .iter()
        .skip(start)
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| *ch == expected)
}

pub(crate) fn previous_non_ws_is(chars: &[char], start: usize, expected: char) -> bool {
    chars
        .iter()
        .take(start)
        .rev()
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| *ch == expected)
}

pub(crate) fn looks_like_jsx_tag(chars: &[char], index: usize) -> bool {
    match chars.get(index + 1) {
        Some('/') => true,
        Some(ch) => ch.is_ascii_alphabetic(),
        None => false,
    }
}

pub(crate) fn scan_jsx_tag(chars: &[char], start: usize) -> usize {
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

pub(crate) fn highlight_jsx_tag(tag: &str) -> String {
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
            let word: String = chars[start..index].iter().collect();
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

pub(crate) fn find_single_char_after(chars: &[char], start: usize, needle: char) -> Option<usize> {
    chars
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, ch)| (*ch == needle).then_some(index))
}

pub(crate) fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

pub(crate) fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

pub(crate) fn paint_maybe(text: &str, color: &str) -> String {
    if color.is_empty() {
        text.to_string()
    } else {
        ansi::paint(text, color)
    }
}
