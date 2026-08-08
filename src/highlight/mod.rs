mod generic;
mod helpers;
mod json;
mod jsx;
mod markdown;
mod plain_text;

use crate::language::Language;

use self::generic::GenericHighlighter;
use self::json::JsonHighlighter;
use self::jsx::JsxHighlighter;
use self::markdown::{MarkdownHighlighter, MdxHighlighter};
use self::plain_text::PlainTextHighlighter;

pub trait SyntaxHighlighter {
    fn highlight_line(&mut self, line: &str) -> String;
}

pub fn create_highlighter(language: Language) -> Box<dyn SyntaxHighlighter> {
    match language {
        Language::PlainText => Box::new(PlainTextHighlighter),
        Language::Json => Box::new(JsonHighlighter),
        Language::JavaScript | Language::TypeScript | Language::Go | Language::Rust
        | Language::Swift | Language::Kotlin | Language::Java => {
            Box::new(GenericHighlighter::new(language))
        }
        Language::Jsx | Language::Tsx => Box::new(JsxHighlighter::new(language)),
        Language::Markdown => Box::new(MarkdownHighlighter::new()),
        Language::Mdx => Box::new(MdxHighlighter::new()),
    }
}

#[cfg(test)]
pub fn highlight_line(line: &str, language: Language) -> String {
    let mut highlighter = create_highlighter(language);
    highlighter.highlight_line(line)
}

#[cfg(test)]
pub fn highlight_document(content: &str, language: Language) -> String {
    let mut highlighter = create_highlighter(language);
    let mut output = String::new();
    for line in content.lines() {
        output.push_str(&highlighter.highlight_line(line));
        output.push('\n');
    }
    output
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
        let highlighted = highlight_line("\"name\": \"scat\"", Language::Json);
        assert!(highlighted.contains(&format!("{}\"name\"{}", ansi::BLUE, ansi::RESET)));
        assert!(highlighted.contains(&format!("{}\"scat\"{}", ansi::GREEN, ansi::RESET)));
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
    fn highlights_markdown_headings() {
        let highlighted = highlight_line("# Title", Language::Markdown);
        assert!(highlighted.contains(&format!("{}#{}", ansi::MAGENTA, ansi::RESET)));
        assert!(highlighted.contains(&format!("{} Title{}", ansi::BLUE, ansi::RESET)));
    }

    #[test]
    fn highlights_markdown_inline_code_and_links() {
        let highlighted = highlight_line(
            "Use `scat` from [docs](https://example.com)",
            Language::Markdown,
        );
        assert!(highlighted.contains(&format!("{}`scat`{}", ansi::GREEN, ansi::RESET)));
        assert!(highlighted.contains(&format!("{}docs{}", ansi::BLUE, ansi::RESET)));
        assert!(highlighted.contains(&format!(
            "{}https://example.com{}",
            ansi::GREEN,
            ansi::RESET
        )));
    }

    #[test]
    fn highlights_markdown_fenced_code_blocks() {
        let highlighted = highlight_document("```rust\nfn main() {}\n```\n", Language::Markdown);
        assert!(highlighted.contains(&format!("{}```rust{}", ansi::BRIGHT_BLACK, ansi::RESET)));
        assert!(highlighted.contains(&format!("{}fn main() {{}}{}", ansi::GREEN, ansi::RESET)));
    }

    #[test]
    fn highlights_mdx_jsx_tags() {
        let highlighted = highlight_line("<Meta title=\"Button\" />", Language::Mdx);
        assert!(highlighted.contains(&format!("{}Meta{}", ansi::MAGENTA, ansi::RESET)));
        assert!(highlighted.contains(&format!("{}title{}", ansi::BLUE, ansi::RESET)));
        assert!(highlighted.contains(&format!("{}\"Button\"{}", ansi::GREEN, ansi::RESET)));
    }

    #[test]
    fn highlights_mdx_imports_as_tsx() {
        let highlighted =
            highlight_line("import { Meta } from '@storybook/blocks';", Language::Mdx);
        assert!(highlighted.contains(&format!("{}import{}", ansi::MAGENTA, ansi::RESET)));
        assert!(highlighted.contains(&format!("{}from{}", ansi::MAGENTA, ansi::RESET)));
        assert!(highlighted.contains(&format!(
            "{}'@storybook/blocks'{}",
            ansi::GREEN,
            ansi::RESET
        )));
    }

    #[test]
    fn highlights_inline_mdx_components_in_markdown() {
        let highlighted = highlight_line("Render <Button size=\"small\" /> here.", Language::Mdx);
        assert!(highlighted.contains(&format!("{}Button{}", ansi::MAGENTA, ansi::RESET)));
        assert!(highlighted.contains(&format!("{}size{}", ansi::BLUE, ansi::RESET)));
    }

    #[test]
    fn leaves_plain_text_unchanged() {
        assert_eq!(
            highlight_line("hello 123", Language::PlainText),
            "hello 123"
        );
    }
}
