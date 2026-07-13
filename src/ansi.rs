pub const RESET: &str = "\x1b[0m";
pub const DIM: &str = "\x1b[2m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";
pub const GREEN: &str = "\x1b[32m";
pub const MAGENTA: &str = "\x1b[35m";
pub const YELLOW: &str = "\x1b[33m";
pub const BRIGHT_BLACK: &str = "\x1b[90m";

pub fn paint(text: &str, color: &str) -> String {
    format!("{color}{text}{RESET}")
}
