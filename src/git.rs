use std::error::Error;
use std::fmt;
use std::io;
use std::process::Command;

use crate::args::DiffConfig;

#[derive(Debug)]
pub enum GitError {
    DiffFailed(String),
    EmptyDiff,
    GitUnavailable(io::Error),
    InvalidTargets,
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiffFailed(message) => write!(f, "git diff failed: {message}"),
            Self::EmptyDiff => write!(f, "no git differences"),
            Self::GitUnavailable(source) => write!(f, "could not execute git: {source}"),
            Self::InvalidTargets => write!(f, "git diff accepts zero, one, or two targets"),
        }
    }
}

impl Error for GitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GitUnavailable(source) => Some(source),
            _ => None,
        }
    }
}

pub fn diff(config: &DiffConfig) -> Result<String, GitError> {
    let args = diff_args(config)?;
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(GitError::GitUnavailable)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(GitError::DiffFailed(stderr));
    }

    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    if diff.is_empty() {
        return Err(GitError::EmptyDiff);
    }

    Ok(diff)
}

pub fn diff_args(config: &DiffConfig) -> Result<Vec<String>, GitError> {
    let mut args = vec!["diff".to_string()];

    if config.staged {
        args.push("--staged".to_string());
    }

    match config.targets.as_slice() {
        [] => {}
        [path] => {
            args.push("--".to_string());
            args.push(path.clone());
        }
        [left, right] => {
            args.push(left.clone());
            args.push(right.clone());
        }
        _ => return Err(GitError::InvalidTargets),
    }

    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::diff_args;
    use crate::args::DiffConfig;

    #[test]
    fn builds_default_diff_args() {
        let args = diff_args(&DiffConfig {
            staged: false,
            targets: Vec::new(),
        })
        .unwrap();
        assert_eq!(args, ["diff"]);
    }

    #[test]
    fn builds_staged_diff_args() {
        let args = diff_args(&DiffConfig {
            staged: true,
            targets: Vec::new(),
        })
        .unwrap();
        assert_eq!(args, ["diff", "--staged"]);
    }

    #[test]
    fn builds_path_limited_diff_args() {
        let args = diff_args(&DiffConfig {
            staged: false,
            targets: vec!["src/main.rs".to_string()],
        })
        .unwrap();
        assert_eq!(args, ["diff", "--", "src/main.rs"]);
    }

    #[test]
    fn builds_ref_range_diff_args() {
        let args = diff_args(&DiffConfig {
            staged: false,
            targets: vec!["HEAD~1".to_string(), "HEAD".to_string()],
        })
        .unwrap();
        assert_eq!(args, ["diff", "HEAD~1", "HEAD"]);
    }
}
