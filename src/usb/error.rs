
#[derive(thiserror::Error, Debug)]
pub enum Error 
{
    #[error("USB enumeration parsing error")]
    UsbParsingError,
    #[error("Failed IOServiceGetMatchingServices")]
    FailedIoService,
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown Error")]
    Unknown,
    #[error("{0}")]
    Generic(String),
    #[error("Not implemented")]
    NotImplemented,
    #[error("Cannot read directory {0:?}: {1}")]
    ReadDir(std::path::PathBuf, std::io::Error),
    #[error("Cannot canonicalize broken symlink for {0:?}: {1}")]
    BadSymlink(std::path::PathBuf, std::io::Error),
    #[error("Cannot read file content from {0:?}: {1}")]
    ReadFile(std::path::PathBuf, std::io::Error),
}