mod ansi;
mod args;
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
        eprintln!("mybat: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = args::Config::parse(std::env::args().skip(1))?;

    if config.help {
        println!("{}", args::help_text());
        return Ok(());
    }

    let color = should_use_color(
        config.color,
        io::stdout().is_terminal(),
        std::env::var_os("NO_COLOR").is_some(),
    );

    let content = match &config.mode {
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
                config.pager,
            );
        }
        args::Mode::Diff(diff_config) => git::diff(diff_config)?,
    };

    write_output(
        &content,
        language::Language::PlainText,
        config.line_numbers,
        color,
        config.pager,
    )
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

fn should_use_color(mode: args::ColorMode, stdout_is_terminal: bool, no_color: bool) -> bool {
    match mode {
        args::ColorMode::Always => !no_color,
        args::ColorMode::Auto => stdout_is_terminal && !no_color,
        args::ColorMode::Never => false,
    }
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
    use super::should_use_color;
    use crate::args::ColorMode;

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
}
