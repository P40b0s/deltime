use super::{error::Error, usb_device_info::{self, UsbDeviceInfo}};
use std::{
    collections::HashMap, ffi::OsStr, mem::size_of, os::windows::ffi::OsStrExt, path::Path, ptr::{null, null_mut}, sync::Arc, time::Duration
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use wmi::{COMLibrary, FilterValue, Variant, WMIConnection, WMIDateTime};
use tokio::sync::{mpsc::Sender, Mutex};

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
    drive_type: u32,
    caption: String,
    description: Option<String>,
    device_id: Option<String>,
    volume_name: Option<String>,
    volume_serial_number: String,
    file_system: Option<String>,
    size: u64,
    free_space: u64,

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
pub async fn enumerate_connected_usb(sender: Sender<UsbDeviceInfo>) -> Result<(), Error>
{
    //let mut sender = sender;
    //let sender1 = Arc::new(sender);
    //let sender2 = Arc::clone(&sender1);
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con)?;
    //let results: Vec<Disks> = wmi_con.query()?;
    let mut filters = HashMap::<String, FilterValue>::new();
    filters.insert("TargetInstance".to_owned(), FilterValue::is_a::<Disks>()?);
    let mut stream = wmi_con.async_filtered_notification::<NewProcessEvent>(&filters, Some(Duration::from_secs(1)))?;
    
    //let event = stream.next().await.unwrap()?;
    
    while let Some(d) = stream.next().await
    {
       
        let result = d?;
        sender.send(result.target_instance.into());
    }
    Ok(())
}