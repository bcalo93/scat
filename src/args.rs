use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::language::Language;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub mode: Mode,
    pub line_numbers: bool,
    pub color: ColorMode,
    pub language: Option<Language>,
    pub pager: bool,
    pub help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    View { path: Option<PathBuf> },
    Diff(DiffConfig),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffConfig {
    pub staged: bool,
    pub targets: Vec<String>,
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
    StagedRequiresDiff,
    TooManyDiffTargets,
    TooManyInputs,
    UnknownFlag(String),
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLanguage(language) => write!(f, "unsupported language '{language}'"),
            Self::MissingValue(flag) => write!(f, "missing value for '{flag}'"),
            Self::StagedRequiresDiff => write!(f, "--staged can only be used with --diff"),
            Self::TooManyDiffTargets => write!(f, "--diff accepts at most two targets"),
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
        let mut diff = false;
        let mut staged = false;
        let mut targets = Vec::new();
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
                "--diff" => diff = true,
                "--staged" => staged = true,
                "-" if !diff => set_path(&mut path, PathBuf::from("-"))?,
                "-" => targets.push(arg),
                _ if arg.starts_with('-') => return Err(ArgsError::UnknownFlag(arg)),
                _ if diff => targets.push(arg),
                _ => set_path(&mut path, PathBuf::from(arg))?,
            }
        }

        if staged && !diff {
            return Err(ArgsError::StagedRequiresDiff);
        }

        if targets.len() > 2 {
            return Err(ArgsError::TooManyDiffTargets);
        }

        let mode = if diff {
            Mode::Diff(DiffConfig { staged, targets })
        } else {
            Mode::View { path }
        };

        Ok(Self {
            mode,
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
    "Usage: scat [OPTIONS] [FILE|-]\n       scat --diff [--staged] [PATH|REF REF]\n\nOptions:\n      --diff              Show git diff output\n      --staged            Show staged git diff output\n  -l, --language <lang>   Force language highlighting\n  -n, --line-numbers      Show line numbers (default)\n      --no-line-numbers   Hide line numbers\n      --color             Force ANSI colors\n      --plain             Disable colors and highlighting\n      --no-color          Disable colors and highlighting\n      --pager             Send output to $PAGER or less -R\n  -h, --help              Show this help\n\nSupported languages: json, js, ts, jsx, tsx, go, rust, swift, kotlin, java, markdown, mdx\nWhen FILE is omitted or '-' is used, scat reads from stdin.\nBy default, color is enabled only for terminal output. Set NO_COLOR to disable ANSI output."
}

#[cfg(test)]
mod tests {
    use super::{ArgsError, ColorMode, Config, DiffConfig, Mode};
    use std::path::PathBuf;

    #[test]
    fn parses_path_and_defaults_to_line_numbers() {
        let config = Config::parse(["src/main.rs"]).unwrap();
        assert_eq!(
            config.mode,
            Mode::View {
                path: Some(PathBuf::from("src/main.rs"))
            }
        );
        assert!(config.line_numbers);
        assert_eq!(config.color, ColorMode::Auto);
        assert_eq!(config.language, None);
        assert!(!config.pager);
    }

    #[test]
    fn parses_no_line_numbers() {
        let config = Config::parse(["--no-line-numbers", "-"]).unwrap();
        assert_eq!(
            config.mode,
            Mode::View {
                path: Some(PathBuf::from("-"))
            }
        );
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
    fn parses_diff_mode() {
        let config = Config::parse(["--diff"]).unwrap();
        assert_eq!(
            config.mode,
            Mode::Diff(DiffConfig {
                staged: false,
                targets: Vec::new()
            })
        );
    }

    #[test]
    fn parses_staged_diff_mode() {
        let config = Config::parse(["--diff", "--staged"]).unwrap();
        assert_eq!(
            config.mode,
            Mode::Diff(DiffConfig {
                staged: true,
                targets: Vec::new()
            })
        );
    }

    #[test]
    fn parses_diff_path_target() {
        let config = Config::parse(["--diff", "src/main.rs"]).unwrap();
        assert_eq!(
            config.mode,
            Mode::Diff(DiffConfig {
                staged: false,
                targets: vec!["src/main.rs".to_string()]
            })
        );
    }

    #[test]
    fn parses_diff_ref_targets() {
        let config = Config::parse(["--diff", "HEAD~1", "HEAD"]).unwrap();
        assert_eq!(
            config.mode,
            Mode::Diff(DiffConfig {
                staged: false,
                targets: vec!["HEAD~1".to_string(), "HEAD".to_string()]
            })
        );
    }

    #[test]
    fn rejects_staged_without_diff() {
        let err = Config::parse(["--staged"]).unwrap_err();
        assert_eq!(err, ArgsError::StagedRequiresDiff);
    }

    #[test]
    fn rejects_too_many_diff_targets() {
        let err = Config::parse(["--diff", "a", "b", "c"]).unwrap_err();
        assert_eq!(err, ArgsError::TooManyDiffTargets);
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
