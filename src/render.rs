use crate::ansi;
use crate::highlight;
use crate::language::Language;

pub fn render(content: &str, language: Language, line_numbers: bool, color: bool) -> String {
    let line_count = content.lines().count().max(1);
    let width = line_count.to_string().len();
    let mut output = String::new();

    for (index, line) in content.lines().enumerate() {
        if line_numbers {
            let number = format!("{:>width$}", index + 1, width = width);
            if color {
                output.push_str(&ansi::paint(&number, ansi::BLUE));
                output.push_str(&ansi::paint(" | ", ansi::DIM));
            } else {
                output.push_str(&number);
                output.push_str(" | ");
            }
        }
        if color {
            output.push_str(&highlight::highlight_line(line, language));
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::render;
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
}
