use crate::ansi;
use crate::highlight;
use crate::language::Language;

pub fn render(content: &str, language: Language, line_numbers: bool) -> String {
    let line_count = content.lines().count().max(1);
    let width = line_count.to_string().len();
    let mut output = String::new();

    for (index, line) in content.lines().enumerate() {
        if line_numbers {
            let number = format!("{:>width$}", index + 1, width = width);
            output.push_str(&ansi::paint(&number, ansi::BLUE));
            output.push_str(&ansi::paint(" | ", ansi::DIM));
        }
        output.push_str(&highlight::highlight_line(line, language));
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
        let rendered = render("one\ntwo\n", Language::PlainText, true);
        assert!(rendered.contains("1"));
        assert!(rendered.contains("2"));
    }

    #[test]
    fn can_hide_line_numbers() {
        let rendered = render("one\n", Language::PlainText, false);
        assert_eq!(rendered, "one\n");
    }
}
