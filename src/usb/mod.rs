mod usb_device_info;

#[cfg(all(target_os = "windows", feature = "usb"))]
mod windows;
#[cfg(all(target_os = "windows", feature = "usb"))]
pub use windows::usb_event;

#[cfg(all(target_os = "linux", feature = "usb"))]
mod linux;
#[cfg(all(target_os = "linux", feature = "usb"))]
mod mountpoints;
#[cfg(all(target_os = "linux", feature = "usb"))]
pub use linux::usb_event;


pub use usb_device_info::UsbDeviceInfo;

