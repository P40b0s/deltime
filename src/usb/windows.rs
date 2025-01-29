use super::{usb_device_info::UsbDeviceInfo, usb_device_info::Usb, error::Error};

impl Usb
{
    pub fn enumerate_connected_usb() -> Result<Vec<UsbDeviceInfo>, Error> 
    {
        let mut output: Vec<UsbDeviceInfo> = Vec::new();
        

        Ok(output)
    }
    
}