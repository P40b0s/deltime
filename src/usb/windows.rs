use crate::error::Error;
use super::usb_device_info::UsbDeviceInfo;
use std::{
    collections::HashMap, path::{Path, PathBuf}, pin::Pin, time::Duration
};
use futures::{Stream, StreamExt};
use serde::Deserialize;
use wmi::{COMLibrary, FilterValue, WMIConnection,  WMIError};

#[derive(Deserialize, Debug)]
#[serde(rename = "__InstanceCreationEvent")]
#[serde(rename_all = "PascalCase")]
struct NewProcessEvent 
{
    target_instance: Disks
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_LogicalDisk")]
#[serde(rename_all = "PascalCase")]
struct Disks 
{
    //2 - removable disk
    #[allow(dead_code)]
    drive_type: u32,
    caption: String,
    description: Option<String>,
    device_id: Option<String>,
    volume_name: Option<String>,
    volume_serial_number: String,
    file_system: Option<String>,
    #[allow(dead_code)]
    size: u64,
    #[allow(dead_code)]
    free_space: u64,

}
impl Into<UsbDeviceInfo> for NewProcessEvent
{
    fn into(self) -> UsbDeviceInfo 
    {
        self.target_instance.into()
    }
}
impl Into<UsbDeviceInfo> for Disks
{
    fn into(self) -> UsbDeviceInfo 
    {
        UsbDeviceInfo
        {
            vendor: None,
            description: self.description,
            serial_number: None,
            volume_label: self.volume_name,
            filesystem: self.file_system,
            dev_name: self.device_id,
            fs_id_uuid: Some(self.volume_serial_number),
            mount_point: Some(Path::new(&[&self.caption, "\\"].concat()).to_path_buf())

        }
    }
}

fn convert_stream(inbond_stream: impl Stream<Item = Result<NewProcessEvent, WMIError>>) -> impl Stream<Item = PathBuf>
{
    let s = inbond_stream.filter_map(|t| 
    {
        async move 
        {
            match t 
            {
                Ok(r) => Some(Path::new(&[&r.target_instance.caption, "\\"].concat()).to_path_buf()),
                Err(_) => None
            }
        }
    });
    s
}
pub fn usb_event() -> Result<Pin<Box<impl Stream<Item = std::path::PathBuf>>>, Error> 
{
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;
    let mut filters = HashMap::<String, FilterValue>::new();
    let value = FilterValue::is_a::<Disks>()?;
    filters.insert("TargetInstance".to_owned(), value);
    wmi_con.async_filtered_notification::<NewProcessEvent>(&filters, Some(Duration::from_secs(2)))
        .map_err(|e| Error::Wmi(e))
        .and_then(|s| Ok(Box::pin(convert_stream(s))))
}