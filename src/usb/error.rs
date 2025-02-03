
#[derive(thiserror::Error, Debug)]
pub enum Error 
{
    #[error("USB enumeration parsing error")]
    UsbParsingError,
    #[error("{0}")]
    Generic(String),
    #[cfg(target_os = "windows")]
    #[error("{0}")]
    Wmi(#[from] wmi::utils::WMIError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}