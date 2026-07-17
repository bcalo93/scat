mod ansi;
mod args;
mod diff_render;
mod git;
mod highlight;
mod input;
mod language;
mod render;

use std::io::{self, IsTerminal};
use std::process;
use std::process::{Command, Stdio};

fn main() {
    if let Err(err) = run() {
        eprintln!("scat: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = args::Config::parse(std::env::args().skip(1))?;

    if config.help {
        println!("{}", args::help_text());
        return Ok(());
    }

    let stdout_is_terminal = io::stdout().is_terminal();
    let color = should_use_color(
        config.color,
        stdout_is_terminal,
        std::env::var_os("NO_COLOR").is_some(),
    );

    match &config.mode {
        args::Mode::View { path } => {
            let source = input::read_source(path.as_deref())?;
            let lang = match config.language {
                Some(lang) => lang,
                None => detect_language(&source),
            };
            return write_output(
                &source.content,
                lang,
                config.line_numbers,
                color,
                should_page_view(&source, config.pager, config.full, stdout_is_terminal),
            );
        }
        args::Mode::Diff(diff_config) => {
            let diff = git::diff(diff_config)?;
            write_diff_output(&diff, config.line_numbers, color, config.pager)
        }
    }
}

fn write_output(
    content: &str,
    language: language::Language,
    line_numbers: bool,
    color: bool,
    pager: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if pager {
        render_to_pager(content, language, line_numbers, color)?;
    } else {
        let mut stdout = io::stdout().lock();
        render::render_to_writer(content, language, line_numbers, color, &mut stdout)?;
    }
    Ok(())
}

fn write_diff_output(
    content: &str,
    line_numbers: bool,
    color: bool,
    pager: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if pager {
        render_diff_to_pager(content, line_numbers, color)?;
    } else {
        let mut stdout = io::stdout().lock();
        diff_render::render_to_writer(content, line_numbers, color, &mut stdout)?;
    }
    Ok(())
}

fn render_to_pager(
    content: &str,
    language: language::Language,
    line_numbers: bool,
    color: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less -R".to_string());
    let mut parts = pager.split_whitespace();
    let program = parts.next().unwrap_or("less");
    let args = parts.collect::<Vec<_>>();

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        render::render_to_writer(content, language, line_numbers, color, &mut stdin)?;
    }

    child.wait()?;
    Ok(())
}

fn render_diff_to_pager(
    content: &str,
    line_numbers: bool,
    color: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let pager = std::env::var("PAGER").unwrap_or_else(|_| "less -R".to_string());
    let mut parts = pager.split_whitespace();
    let program = parts.next().unwrap_or("less");
    let args = parts.collect::<Vec<_>>();

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        diff_render::render_to_writer(content, line_numbers, color, &mut stdin)?;
    }

    child.wait()?;
    Ok(())
}

fn should_use_color(mode: args::ColorMode, stdout_is_terminal: bool, no_color: bool) -> bool {
    match mode {
        args::ColorMode::Always => !no_color,
        args::ColorMode::Auto => stdout_is_terminal && !no_color,
        args::ColorMode::Never => false,
    }
}

fn should_page_view(
    source: &input::Source,
    explicit_pager: bool,
    full: bool,
    stdout_is_terminal: bool,
) -> bool {
    !full && (explicit_pager || (stdout_is_terminal && source.path.is_some()))
}

fn detect_language(source: &input::Source) -> language::Language {
    match source
        .path
        .as_deref()
        .and_then(language::Language::from_path)
    {
        Some(lang) => lang,
        None if source.path.is_none() => language::Language::infer_from_content(&source.content),
        None => language::Language::PlainText,
    }
}

#[cfg(test)]
mod tests {
    use super::{should_page_view, should_use_color};
    use crate::args::ColorMode;
    use crate::input::Source;
    use std::path::PathBuf;

    #[test]
    fn color_auto_depends_on_terminal() {
        assert!(should_use_color(ColorMode::Auto, true, false));
        assert!(!should_use_color(ColorMode::Auto, false, false));
    }

    #[test]
    fn color_modes_can_force_or_disable_color() {
        assert!(should_use_color(ColorMode::Always, false, false));
        assert!(!should_use_color(ColorMode::Never, true, false));
    }

    #[test]
    fn no_color_overrides_color_modes() {
        assert!(!should_use_color(ColorMode::Always, true, true));
        assert!(!should_use_color(ColorMode::Auto, true, true));
    }

    #[test]
    fn files_page_by_default_on_terminal() {
        let source = Source {
            path: Some(PathBuf::from("src/main.rs")),
            content: "fn main() {}\n".to_string(),
        };

        assert!(should_page_view(&source, false, false, true));
    }

    #[test]
    fn full_disables_default_file_pager() {
        let source = Source {
            path: Some(PathBuf::from("src/main.rs")),
            content: "fn main() {}\n".to_string(),
        };

        assert!(!should_page_view(&source, false, true, true));
    }

    #[test]
    fn stdin_and_redirected_output_do_not_page_by_default() {
        let stdin_source = Source {
            path: None,
            content: "fn main() {}\n".to_string(),
        };
        let file_source = Source {
            path: Some(PathBuf::from("src/main.rs")),
            content: "fn main() {}\n".to_string(),
        };

        assert!(!should_page_view(&stdin_source, false, false, true));
        assert!(!should_page_view(&file_source, false, false, false));
    }

    #[test]
    fn explicit_pager_still_pages_without_full() {
        let source = Source {
            path: None,
            content: "fn main() {}\n".to_string(),
        };

        assert!(should_page_view(&source, true, false, false));
    }

    #[test]
    fn full_overrides_explicit_pager() {
        let source = Source {
            path: Some(PathBuf::from("src/main.rs")),
            content: "fn main() {}\n".to_string(),
        };

        assert!(!should_page_view(&source, true, true, true));
    }
}
