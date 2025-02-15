use crate::structs::TaskWithProgress;
use crate::error::Error;
use super::{usb_device_info::{self, UsbDeviceInfo}};
use std::{
    collections::HashMap, ffi::OsStr, mem::size_of, os::windows::ffi::OsStrExt, path::{Path, PathBuf}, pin::Pin, ptr::{null, null_mut}, sync::Arc, time::Duration
};
use futures::{stream, SinkExt, Stream, StreamExt};
use scheduler::Scheduler;
use serde::Deserialize;
use wmi::{COMLibrary, FilterValue, Variant, WMIConnection, WMIDateTime, WMIError};
use tokio::{runtime::Handle, sync::{mpsc::{Receiver, Sender}, Mutex, RwLock}, task};

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
fn convert_stream_old(inbond_stream: impl Stream<Item = Result<NewProcessEvent, WMIError>>) -> impl Stream<Item = UsbDeviceInfo>
{
    let s = inbond_stream.filter_map(|t| 
    {
        async move 
        {
            match t 
            {
                Ok(r) => Some(r.into()),
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

//pub async fn enumerate_connected_usb<F: Send, R>(closure: F) -> Result<(), Error> where F: Fn(UsbDeviceInfo) -> R, R: std::future::Future<Output = ()> 
// pub async fn enumerate_connected_usb() -> Result<impl Stream<Item = UsbDeviceInfo>, Error>
// {
//     //let mut sender = sender;
//     //let sender1 = Arc::new(sender);
//     //let sender2 = Arc::clone(&sender1);
//     let com_con = COMLibrary::new()?;
//     let wmi_con = WMIConnection::new(com_con)?;
//     let mut filters = HashMap::<String, FilterValue>::new();
//     if let Ok(value) = FilterValue::is_a::<Disks>()
//     {
//         filters.insert("TargetInstance".to_owned(), value);
//         if let Ok(stream) = wmi_con.async_filtered_notification::<NewProcessEvent>(&filters, Some(Duration::from_secs(2)))
//         {
//             return Ok(convert_stream(stream));
//         }
//     }
//     // if let Ok(com_con) = COMLibrary::new()
//     // {
//     //     if let Ok(wmi_con) = WMIConnection::new(com_con) 
//     //     {
//     //         let mut filters = HashMap::<String, FilterValue>::new();
//     //         if let Ok(value) = FilterValue::is_a::<Disks>()
//     //         {
//     //             filters.insert("TargetInstance".to_owned(), value);
//     //             if let Ok(stream) = wmi_con.async_filtered_notification::<NewProcessEvent>(&filters, Some(Duration::from_secs(2)))
//     //             {
//     //                 return Some(convert_stream(stream));
//     //             }
//     //         }
//     //     }
//     // }
//     // None
    
// }

// pub async fn enumerate_connected_usb(sender: Sender<UsbDeviceInfo>) -> impl Stream<Item = Option<UsbDeviceInfo>>
// {
//     //let mut sender = sender;
//     //let sender1 = Arc::new(sender);
//     //let sender2 = Arc::clone(&sender1);
//     if let com_con = COMLibrary::new()?;
//     let wmi_con = WMIConnection::new(com_con)?;
//     //let results: Vec<Disks> = wmi_con.query()?;
//     let mut filters = HashMap::<String, FilterValue>::new();
//     filters.insert("TargetInstance".to_owned(), FilterValue::is_a::<Disks>()?);
//     let mut stream = wmi_con.async_filtered_notification::<NewProcessEvent>(&filters, Some(Duration::from_secs(2)))?;
//     //let event = stream.next().await.unwrap()?;
//     // let s = stream.map(|t| 
//     // {
//     //     match t 
//     //     {
//     //         Ok(r) => Some(r.into()),
//     //         Err(_) => None
//     //     }
//     // });
//     //let mut b: impl Stream<Item = Option<UsbDeviceInfo>> = s;
//     //let mut b = Box::pin(s) as Pin<Box<dyn Stream<Item = Option<UsbDeviceInfo>>>>;
//     //while let Some(ss) = b.next().await
//     //{

//     //}
//     // while let Some(d) = stream.next().await
//     // {
//     //     logger::info!("result in windows.rs: {:?}", &d);
//     //     let result = d?;
//     //     let s = sender.send(result.into()).await;
//     //     logger::info!("result in windows.rs sender result: {:?}", s);
//     // }
//     //Ok(())
//     convert_stream(stream)
// }


// pub async fn enumerate(tasks: Arc<RwLock<HashMap<uuid::Uuid, TaskWithProgress>>>, scheduler: Arc<Scheduler<uuid::Uuid>>)
// {
//     //TODO проверить этот вариант, возможно не хочет потому что запускалось из futures::ececutor::block_on
//     tokio::task::block_in_place(move ||
//     {
//         Handle::current().block_on(async move 
//         {
//             if let Some(stream) = enumerate_connected_usb().await
//             {
//                 let mut stream = Box::pin(stream);
//                 while let Some(device) = stream.next().await
//                 {
//                     logger::info!("result from enumerate_connected_usb in main: {:?}", device);
//                 }
//             }
//         }); 
//     });
// }