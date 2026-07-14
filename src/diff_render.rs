use std::io::{self, Write};

use crate::ansi;

#[cfg(test)]
pub fn render(content: &str, line_numbers: bool, color: bool) -> String {
    let mut output = Vec::new();
    render_to_writer(content, line_numbers, color, &mut output)
        .expect("rendering diff to memory should not fail");
    String::from_utf8(output).expect("diff renderer writes valid UTF-8")
}

pub fn render_to_writer<W: Write>(
    content: &str,
    line_numbers: bool,
    color: bool,
    writer: &mut W,
) -> io::Result<()> {
    let line_count = content.lines().count().max(1);
    let width = line_count.to_string().len();

    for (index, line) in content.lines().enumerate() {
        if line_numbers {
            write_line_number(index + 1, width, color, writer)?;
        }

        if color {
            write!(writer, "{}", highlight_diff_line(line))?;
        } else {
            write!(writer, "{line}")?;
        }
        writeln!(writer)?;
    }

    Ok(())
}

fn write_line_number<W: Write>(
    number: usize,
    width: usize,
    color: bool,
    writer: &mut W,
) -> io::Result<()> {
    let formatted = format!("{number:>width$}");
    if color {
        write!(writer, "{}", ansi::paint(&formatted, ansi::BLUE))?;
        write!(writer, "{}", ansi::paint(" | ", ansi::DIM))?;
    } else {
        write!(writer, "{formatted} | ")?;
    }
    Ok(())
}

fn highlight_diff_line(line: &str) -> String {
    if line.starts_with("diff --git ") {
        return ansi::paint(line, ansi::BLUE);
    }

    if line.starts_with("index ")
        || line.starts_with("new file mode ")
        || line.starts_with("deleted file mode ")
    {
        return ansi::paint(line, ansi::BRIGHT_BLACK);
    }

    if line.starts_with("@@") {
        return ansi::paint(line, ansi::CYAN);
    }

    if line.starts_with("+++") {
        return ansi::paint(line, ansi::GREEN);
    }

    if line.starts_with("---") {
        return ansi::paint(line, ansi::MAGENTA);
    }

    if line.starts_with('+') {
        return ansi::paint(line, ansi::GREEN);
    }

    if line.starts_with('-') {
        return ansi::paint(line, ansi::MAGENTA);
    }

    line.to_string()
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::ansi;

    #[test]
    fn highlights_diff_headers() {
        let rendered = render(
            "diff --git a/a.rs b/a.rs\nindex abc..def 100644\n",
            false,
            true,
        );
        assert!(rendered.contains(&format!(
            "{}diff --git a/a.rs b/a.rs{}",
            ansi::BLUE,
            ansi::RESET
        )));
        assert!(rendered.contains(&format!(
            "{}index abc..def 100644{}",
            ansi::BRIGHT_BLACK,
            ansi::RESET
        )));
    }

    #[test]
    fn highlights_hunks() {
        let rendered = render("@@ -1,2 +1,2 @@\n", false, true);
        assert!(rendered.contains(&format!("{}@@ -1,2 +1,2 @@{}", ansi::CYAN, ansi::RESET)));
    }

    #[test]
    fn highlights_added_and_removed_lines() {
        let rendered = render("+new\n-old\n", false, true);
        assert!(rendered.contains(&format!("{}+new{}", ansi::GREEN, ansi::RESET)));
        assert!(rendered.contains(&format!("{}-old{}", ansi::MAGENTA, ansi::RESET)));
    }

    #[test]
    fn treats_file_markers_as_headers() {
        let rendered = render("--- a/src/main.rs\n+++ b/src/main.rs\n", false, true);
        assert!(rendered.contains(&format!(
            "{}--- a/src/main.rs{}",
            ansi::MAGENTA,
            ansi::RESET
        )));
        assert!(rendered.contains(&format!("{}+++ b/src/main.rs{}", ansi::GREEN, ansi::RESET)));
    }

    #[test]
    fn can_disable_color() {
        let rendered = render("+new\n", false, false);
        assert_eq!(rendered, "+new\n");
    }

    #[test]
    fn can_show_line_numbers() {
        let rendered = render("+new\n-old\n", true, false);
        assert_eq!(rendered, "1 | +new\n2 | -old\n");
    }
}
