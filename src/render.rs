use std::io::{self, Write};

use crate::ansi;
use crate::highlight::create_highlighter;
use crate::language::Language;

#[cfg(test)]
pub fn render(content: &str, language: Language, line_numbers: bool, color: bool) -> String {
    let mut output = Vec::new();
    render_to_writer(content, language, line_numbers, color, &mut output)
        .expect("rendering to memory should not fail");
    String::from_utf8(output).expect("renderer writes valid UTF-8")
}

pub fn render_to_writer<W: Write>(
    content: &str,
    language: Language,
    line_numbers: bool,
    color: bool,
    writer: &mut W,
) -> io::Result<()> {
    let line_count = content.lines().count().max(1);
    let width = line_count.to_string().len();
    let mut highlighter = create_highlighter(language);

    for (index, line) in content.lines().enumerate() {
        if line_numbers {
            let number = format!("{:>width$}", index + 1, width = width);
            if color {
                write!(writer, "{}", ansi::paint(&number, ansi::BLUE))?;
                write!(writer, "{}", ansi::paint(" | ", ansi::DIM))?;
            } else {
                write!(writer, "{number} | ")?;
            }
        }
        if color {
            write!(writer, "{}", highlighter.highlight_line(line))?;
        } else {
            write!(writer, "{line}")?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{render, render_to_writer};
    use crate::language::Language;

    #[test]
    fn renders_line_numbers() {
        let rendered = render("one\ntwo\n", Language::PlainText, true, true);
        assert!(rendered.contains("1"));
        assert!(rendered.contains("2"));
    }

    #[test]
    fn can_hide_line_numbers() {
        let rendered = render("one\n", Language::PlainText, false, true);
        assert_eq!(rendered, "one\n");
    }

    #[test]
    fn can_disable_color() {
        let rendered = render("fn main() {}\n", Language::Rust, false, false);
        assert_eq!(rendered, "fn main() {}\n");
    }

    #[test]
    fn can_render_to_writer() {
        let mut output = Vec::new();
        render_to_writer("one\n", Language::PlainText, true, false, &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "1 | one\n");
    }
}
