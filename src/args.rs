use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::language::Language;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub path: Option<PathBuf>,
    pub line_numbers: bool,
    pub color: ColorMode,
    pub language: Option<Language>,
    pub pager: bool,
    pub help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgsError {
    InvalidLanguage(String),
    MissingValue(&'static str),
    TooManyInputs,
    UnknownFlag(String),
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLanguage(language) => write!(f, "unsupported language '{language}'"),
            Self::MissingValue(flag) => write!(f, "missing value for '{flag}'"),
            Self::TooManyInputs => write!(f, "too many input paths; pass a single file or '-'"),
            Self::UnknownFlag(flag) => write!(f, "unknown flag '{flag}'"),
        }
    }
}

impl Error for ArgsError {}

impl Config {
    pub fn parse<I, S>(args: I) -> Result<Self, ArgsError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut path = None;
        let mut line_numbers = true;
        let mut color = ColorMode::Auto;
        let mut language = None;
        let mut pager = false;
        let mut help = false;
        let mut iter = args.into_iter();

        while let Some(raw) = iter.next() {
            let arg = raw.into();
            match arg.as_str() {
                "-h" | "--help" => help = true,
                "-l" | "--language" => {
                    let value = iter
                        .next()
                        .map(Into::into)
                        .ok_or(ArgsError::MissingValue("--language"))?;
                    language = Some(
                        Language::from_name(&value)
                            .ok_or_else(|| ArgsError::InvalidLanguage(value.clone()))?,
                    );
                }
                "-n" | "--line-numbers" => line_numbers = true,
                "--no-line-numbers" => line_numbers = false,
                "--color" => color = ColorMode::Always,
                "--plain" | "--no-color" => color = ColorMode::Never,
                "--pager" => pager = true,
                "-" => set_path(&mut path, PathBuf::from("-"))?,
                _ if arg.starts_with('-') => return Err(ArgsError::UnknownFlag(arg)),
                _ => set_path(&mut path, PathBuf::from(arg))?,
            }
        }

        Ok(Self {
            path,
            line_numbers,
            color,
            language,
            pager,
            help,
        })
    }
}

fn set_path(path: &mut Option<PathBuf>, value: PathBuf) -> Result<(), ArgsError> {
    if path.is_some() {
        return Err(ArgsError::TooManyInputs);
    }
    *path = Some(value);
    Ok(())
}

pub fn help_text() -> &'static str {
    "Usage: mybat [OPTIONS] [FILE|-]\n\nOptions:\n  -l, --language <lang>   Force language highlighting\n  -n, --line-numbers      Show line numbers (default)\n      --no-line-numbers   Hide line numbers\n      --color             Force ANSI colors\n      --plain             Disable colors and highlighting\n      --no-color          Disable colors and highlighting\n      --pager             Send output to $PAGER or less -R\n  -h, --help              Show this help\n\nSupported languages: json, js, ts, jsx, tsx, go, rust, swift, kotlin, java\nWhen FILE is omitted or '-' is used, mybat reads from stdin.\nBy default, color is enabled only for terminal output. Set NO_COLOR to disable ANSI output."
}

#[cfg(test)]
mod tests {
    use super::{ArgsError, ColorMode, Config};
    use std::path::PathBuf;

    #[test]
    fn parses_path_and_defaults_to_line_numbers() {
        let config = Config::parse(["src/main.rs"]).unwrap();
        assert_eq!(config.path, Some(PathBuf::from("src/main.rs")));
        assert!(config.line_numbers);
        assert_eq!(config.color, ColorMode::Auto);
        assert_eq!(config.language, None);
        assert!(!config.pager);
    }

    #[test]
    fn parses_no_line_numbers() {
        let config = Config::parse(["--no-line-numbers", "-"]).unwrap();
        assert_eq!(config.path, Some(PathBuf::from("-")));
        assert!(!config.line_numbers);
    }

    #[test]
    fn parses_forced_language() {
        let config = Config::parse(["--language", "typescript", "-"]).unwrap();
        assert_eq!(config.language, Some(crate::language::Language::TypeScript));
    }

    #[test]
    fn parses_plain_output() {
        let config = Config::parse(["--plain", "src/main.rs"]).unwrap();
        assert_eq!(config.color, ColorMode::Never);
    }

    #[test]
    fn parses_forced_color() {
        let config = Config::parse(["--color", "src/main.rs"]).unwrap();
        assert_eq!(config.color, ColorMode::Always);
    }

    #[test]
    fn parses_pager() {
        let config = Config::parse(["--pager", "src/main.rs"]).unwrap();
        assert!(config.pager);
    }

    #[test]
    fn rejects_missing_language_value() {
        let err = Config::parse(["--language"]).unwrap_err();
        assert_eq!(err, ArgsError::MissingValue("--language"));
    }

    #[test]
    fn rejects_invalid_language() {
        let err = Config::parse(["--language", "ruby"]).unwrap_err();
        assert_eq!(err, ArgsError::InvalidLanguage("ruby".to_string()));
    }

    #[test]
    fn rejects_unknown_flags() {
        let err = Config::parse(["--theme"]).unwrap_err();
        assert_eq!(err, ArgsError::UnknownFlag("--theme".to_string()));
    }
}
