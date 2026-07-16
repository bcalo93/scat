use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub path: Option<PathBuf>,
    pub content: String,
}

#[derive(Debug)]
pub enum InputError {
    EmptyInput,
    InvalidPath(PathBuf),
    ReadFailed { path: PathBuf, source: io::Error },
    StdinReadFailed(io::Error),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "input is empty"),
            Self::InvalidPath(path) => write!(f, "'{}' is not a regular file", path.display()),
            Self::ReadFailed { path, source } => {
                write!(f, "could not read '{}': {source}", path.display())
            }
            Self::StdinReadFailed(source) => write!(f, "could not read stdin: {source}"),
        }
    }
}

impl Error for InputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadFailed { source, .. } => Some(source),
            Self::StdinReadFailed(source) => Some(source),
            _ => None,
        }
    }
}

pub fn read_source(path: Option<&Path>) -> Result<Source, InputError> {
    match path {
        Some(path) if path == Path::new("-") => read_stdin(),
        Some(path) => read_file(path),
        None => read_stdin(),
    }
}

fn read_file(path: &Path) -> Result<Source, InputError> {
    let metadata = fs::metadata(path).map_err(|source| InputError::ReadFailed {
        path: path.to_path_buf(),
        source,
    })?;

    if !metadata.is_file() {
        return Err(InputError::InvalidPath(path.to_path_buf()));
    }

    let content = fs::read_to_string(path).map_err(|source| InputError::ReadFailed {
        path: path.to_path_buf(),
        source,
    })?;

    if content.is_empty() {
        return Err(InputError::EmptyInput);
    }

    Ok(Source {
        path: Some(path.to_path_buf()),
        content,
    })
}

fn read_stdin() -> Result<Source, InputError> {
    let mut content = String::new();
    io::stdin()
        .read_to_string(&mut content)
        .map_err(InputError::StdinReadFailed)?;

    if content.is_empty() {
        return Err(InputError::EmptyInput);
    }

    Ok(Source {
        path: None,
        content,
    })
}

#[cfg(test)]
mod tests {
    use super::read_source;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn reads_regular_file() {
        let path = temp_file_path("scat-input-test.txt");
        fs::write(&path, "hello\n").unwrap();

        let source = read_source(Some(&path)).unwrap();

        assert_eq!(source.path, Some(path.clone()));
        assert_eq!(source.content, "hello\n");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_empty_file() {
        let path = temp_file_path("scat-empty-input-test.txt");
        fs::write(&path, "").unwrap();

        let err = read_source(Some(&path)).unwrap_err();

        assert_eq!(err.to_string(), "input is empty");

        fs::remove_file(path).unwrap();
    }

    fn temp_file_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("{}-{name}", std::process::id()));
        path
    }
}
