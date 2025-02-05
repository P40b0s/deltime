mod error;
mod usb_device_info;
#[cfg(all(target_os = "linux", feature = "usb"))]
mod linux;
#[cfg(all(target_os = "linux", feature = "usb"))]
mod mountpoints;
#[cfg(all(target_os = "windows", feature = "winusb"))]
mod windows;

#[cfg(all(target_os = "windows", feature = "winusb"))]
pub use windows::enumerate_connected_usb;
#[cfg(all(target_os = "linux", feature = "usb"))]
pub use linux::enumerate_connected_usb;

pub use usb_device_info::UsbDeviceInfo;

