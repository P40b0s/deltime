mod error;
mod usb_device_info;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
mod mountpoints;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::enumerate_connected_usb;

#[cfg(target_os = "linux")]
pub use linux::enumerate_connected_usb;

pub use usb_device_info::UsbDeviceInfo;

