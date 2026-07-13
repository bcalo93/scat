use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub path: Option<PathBuf>,
    pub line_numbers: bool,
    pub help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgsError {
    TooManyInputs,
    UnknownFlag(String),
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
        let mut help = false;

        for raw in args {
            let arg = raw.into();
            match arg.as_str() {
                "-h" | "--help" => help = true,
                "-n" | "--line-numbers" => line_numbers = true,
                "--no-line-numbers" => line_numbers = false,
                "-" => set_path(&mut path, PathBuf::from("-"))?,
                _ if arg.starts_with('-') => return Err(ArgsError::UnknownFlag(arg)),
                _ => set_path(&mut path, PathBuf::from(arg))?,
            }
        }

        Ok(Self {
            path,
            line_numbers,
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
    "Usage: mybat [OPTIONS] [FILE|-]\n\nOptions:\n  -n, --line-numbers      Show line numbers (default)\n      --no-line-numbers   Hide line numbers\n  -h, --help              Show this help\n\nWhen FILE is omitted or '-' is used, mybat reads from stdin."
}

#[cfg(test)]
mod tests {
    use super::{ArgsError, Config};
    use std::path::PathBuf;

    #[test]
    fn parses_path_and_defaults_to_line_numbers() {
        let config = Config::parse(["src/main.rs"]).unwrap();
        assert_eq!(config.path, Some(PathBuf::from("src/main.rs")));
        assert!(config.line_numbers);
    }

    #[test]
    fn parses_no_line_numbers() {
        let config = Config::parse(["--no-line-numbers", "-"]).unwrap();
        assert_eq!(config.path, Some(PathBuf::from("-")));
        assert!(!config.line_numbers);
    }

    #[test]
    fn rejects_unknown_flags() {
        let err = Config::parse(["--theme"]).unwrap_err();
        assert_eq!(err, ArgsError::UnknownFlag("--theme".to_string()));
    }
}
