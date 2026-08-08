use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
struct Highlighter {
    language: String,
    in_block_comment: bool,
}

impl Highlighter {
    fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
            in_block_comment: false,
        }
    }

    fn highlight_line(&mut self, line: &str) -> String {
        if self.in_block_comment {
            if let Some(end) = line.find("*/") {
                self.in_block_comment = false;
                return format!("{}{}",&line[..end + 2], &line[end + 2..]);
            }
            return line.to_string();
        }

        if line.starts_with("//") {
            return line.to_string();
        }

        if let Some(start) = line.find("/*") {
            if line[start + 2..].find("*/").is_none() {
                self.in_block_comment = true;
            }
            return line.to_string();
        }

        line.to_string()
    }
}

impl fmt::Display for Highlighter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Highlighter({})", self.language)
    }
}

trait SyntaxEngine {
    fn process(&self, input: &str) -> Vec<String>;
}

struct Token {
    kind: TokenKind,
    value: String,
}

enum TokenKind {
    Keyword,
    String,
    Number,
    Comment,
    Plain,
}

fn main() {
    let mut map: HashMap<&str, i32> = HashMap::new();
    map.insert("rust", 1);
    map.insert("go", 2);

    let keys: Vec<_> = map.keys().collect();
    println!("Languages: {:?}", keys);

    let mut h = Highlighter::new("rust");
    let result = h.highlight_line("fn main() {}");
    println!("{}", result);

    let _ = match result.as_str() {
        "fn main() {}" => true,
        _ => false,
    };
}

/* Block comment
   spans multiple lines
*/
