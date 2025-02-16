
#[derive(thiserror::Error, Debug)]
pub enum Error 
{
    #[allow(dead_code)]
    #[error("USB enumeration parsing error")]
    UsbParsingError,
    #[error("{0}")]
    #[allow(dead_code)]
    Generic(String),
    #[cfg(target_os = "windows")]
    #[error("{0}")]
    Wmi(#[from] wmi::utils::WMIError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Utilites(#[from] utilites::error::Error),
}