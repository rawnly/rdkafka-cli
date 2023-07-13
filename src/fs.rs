use serde::{de, Serialize};
use std::{fmt, io, path::Path};

pub type Result<T> = std::result::Result<T, FileError>;

#[derive(Debug, Clone)]
struct PackageJsonError;

#[derive(Debug, Clone)]
pub enum FileKind {
    JSON,
    YAML,
}

impl FileKind {
    pub fn from_path(p: &Path) -> std::result::Result<FileKind, FileError> {
        tracing::debug!("FileKind::from_path({:?})", p);

        if p.extension().is_none() {
            return Err(FileError::NotFound);
        }

        match p.extension().unwrap().to_str().unwrap() {
            "json" => Ok(FileKind::JSON),
            "yml" | "yaml" => Ok(FileKind::YAML),
            ext => Err(FileError::UnsupportedExtension(ext.to_string())),
        }
    }
}

#[async_trait::async_trait]
pub trait File: de::DeserializeOwned + Serialize {
    /// Reads file from filesystem. It must be json or yaml.
    async fn load(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let kind = FileKind::from_path(path)?;

        match kind {
            FileKind::JSON => match serde_json::from_str(&content) {
                Ok(d) => Ok(d),
                Err(e) => Err(FileError::from(e)),
            },
            FileKind::YAML => match serde_yaml::from_str(&content) {
                Ok(d) => Ok(d),
                Err(e) => Err(FileError::from(e)),
            },
        }
    }

    async fn write(&self, path: &Path) -> Result<()> {
        let kind = FileKind::from_path(path)?;

        let contents = match kind {
            FileKind::JSON => serde_json::to_string_pretty(self)?,
            FileKind::YAML => serde_yaml::to_string(self)?,
        };

        tokio::fs::write(path, contents).await?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum FileError {
    NotFound,
    UnsupportedExtension(String),
    InvalidYAMLSyntax(serde_yaml::Error),
    InvalidJSONSyntax(serde_json::Error),
    IO(io::Error),
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message: String = match self {
            FileError::NotFound => "No such file or directory".into(),
            FileError::UnsupportedExtension(extension) => {
                format!("unsupported file extension {extension} (allowed: yaml | yml | json)")
            }
            FileError::InvalidYAMLSyntax(err) => format!("Invalid YAML syntax: {:?}", err),
            FileError::InvalidJSONSyntax(err) => format!("Invalid JSON syntax: {:?}", err),
            FileError::IO(err) => format!("{}", err),
        };

        write!(f, "{}", message)
    }
}

impl From<serde_json::Error> for FileError {
    fn from(e: serde_json::Error) -> Self {
        FileError::InvalidJSONSyntax(e)
    }
}

impl From<serde_yaml::Error> for FileError {
    fn from(e: serde_yaml::Error) -> Self {
        FileError::InvalidYAMLSyntax(e)
    }
}

impl From<io::Error> for FileError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::NotFound => FileError::NotFound,
            _ => FileError::IO(e),
        }
    }
}
