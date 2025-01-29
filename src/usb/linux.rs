
use super::{error::Error, usb_device_info::{self, UsbDeviceInfo}};
use udev::Enumerator;


impl usb_device_info::Usb
{
    pub fn enumerate_connected_usb() -> Result<Vec<UsbDeviceInfo>, Error> 
    {
        let mut output = Vec::new();
        let mut enumerator = match Enumerator::new()
        {
            Ok(res) => res,
            Err(_) => return Err(Error::Generic("could not get udev enumerator".to_string())),
        };
        let mount_points = super::mountpoints::MountPoints::load()?;
        for device in enumerator.scan_devices().expect("could not scan devices") 
        {
            if !is_usb_device(&device) 
            {
                continue;
            }
            let _ = || -> Result<(), Box<dyn std::error::Error>> 
            {
                let _id = device
                    .property_value("DEVPATH")
                    .ok_or(Error::UsbParsingError)?
                    .to_str()
                    .ok_or(Error::UsbParsingError)?
                    .to_string();
    
                let dev_name = device
                .property_value("DEVNAME")
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());
    
                let mut description = device
                    .property_value("ID_MODEL_FROM_DATABASE")
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
    
                if description.is_none() 
                {
                    description = device
                        .property_value("ID_MODEL")
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string());
                }
    
                let vendor = device
                    .property_value("ID_VENDOR")
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
    
                let mut serial_number = device
                    .property_value("ID_SERIAL_SHORT")
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
    
                if serial_number.is_none() 
                {
                    serial_number = device
                        .property_value("ID_SERIAL")
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string());
                }
    
                let volume_label = device
                    .property_value("ID_FS_LABEL")
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
    
                let filesystem = device
                    .property_value("ID_FS_TYPE")
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
    
                let fs_id_uuid = device
                    .property_value("ID_FS_UUID")
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());
                let mount_point = if let Some(n) = dev_name.as_ref()
                {
                    mount_points.get_mount_point(n)
                }
                else
                {
                    None
                };
    
                output.push(UsbDeviceInfo 
                {
                    vendor,
                    description,
                    serial_number,
                    volume_label,
                    filesystem,
                    dev_name,
                    fs_id_uuid,
                    mount_point,
                });
    
                Ok(())
            }();
        }
    
        // remove if filesystem is none, just to stick with only usb drives
        output.retain(|item| !matches!(&item.filesystem.clone().is_none(), true));
        Ok(output)
    }
}



fn is_usb_device(device: &udev::Device) -> bool 
{
    //! Checks if a device is a USB device
    let device_filename = device.devpath().to_string_lossy().to_string();
    let device_subsystem = device.subsystem().unwrap_or_default();
    device_subsystem.eq_ignore_ascii_case("block") && device_filename.contains("/usb")
}


#[cfg(test)]
mod tests
{
    use logger::info;
    use mountpoints::mountpaths;
    use super::usb_device_info::Usb;


    #[test]
    fn test_usb()
    {
        match Usb::enumerate_connected_usb() 
        {
            Ok(l) => 
            {
                if l.is_empty() 
                {
                    println!("No currently connected usb drives")
                } 
                else 
                {
                    println!("{:#?}", l)
                    
                }
            }
            Err(e) => println!("{:?}", e),
        };
        
    }
    #[test]
    fn test_udev()
    {
        logger::StructLogger::new_default();
        let mut enumerator = udev::Enumerator::new().unwrap();
        for device in enumerator.scan_devices().unwrap() {
            info!("");
            info!("{:#?}", device);

            info!("  [properties]");
            for property in device.properties() {
                info!("    - {:?} {:?}", property.name(), property.value());
            }

            info!("  [attributes]");
            for attribute in device.attributes() {
                info!("    - {:?} {:?}", attribute.name(), attribute.value());
            }
        }

    }
}