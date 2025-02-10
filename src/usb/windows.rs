use super::{error::Error, usb_device_info::{self, UsbDeviceInfo}};
use std::{
    collections::HashMap, ffi::OsStr, mem::size_of, os::windows::ffi::OsStrExt, path::Path, ptr::{null, null_mut}, sync::Arc, time::Duration
};
use futures::{SinkExt, Stream, StreamExt};
use serde::Deserialize;
use wmi::{COMLibrary, FilterValue, Variant, WMIConnection, WMIDateTime, WMIError};
use tokio::{sync::{mpsc::Sender, Mutex}, task};

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
//pub async fn enumerate_connected_usb<F: Send, R>(closure: F) -> Result<(), Error> where F: Fn(UsbDeviceInfo) -> R, R: std::future::Future<Output = ()> 
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
    let mut stream = wmi_con.async_filtered_notification::<NewProcessEvent>(&filters, Some(Duration::from_secs(2)))?;
    //let event = stream.next().await.unwrap()?;
    while let Some(d) = stream.next().await
    {
        logger::info!("result in windows.rs: {:?}", &d);
        let result = d?;
        let s = sender.send(result.into()).await;
        logger::info!("result in windows.rs sender result: {:?}", s);
    }
    Ok(())
}


pub async fn enumerate()
{
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<UsbDeviceInfo>(1);
    tokio::spawn(async move
    {
        tokio::spawn(async move 
        {
            futures::executor::block_on(async move
            {
                let r = enumerate_connected_usb(sender).await;
                logger::info!("result from enumerate_connected_usb in main: {:?}", r);
                
            })
        });
        while let Some(stream) = receiver.recv().await
        {
            logger::info!("info from receiver: {:?}", stream);
        }
    });
}