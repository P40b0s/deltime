
use std::{collections::HashMap, future::Future, ops::Deref, path::PathBuf, pin::Pin, sync::Arc};
use crate::structs::TaskWithProgress;
use crate::error::Error;

use super::{usb_device_info::{self, DeviceInfo, UsbDeviceInfo}};
use futures::{stream, Stream};
use scheduler::Scheduler;
use udev::{mio::{Events, Interest, Poll, Token}, Device, Enumerator, Event, EventType};
use tokio::sync::{mpsc::{Receiver, Sender}, Mutex, RwLock};
use utilites::retry_sync;

fn enumerate_connected_usb() -> Result<Vec<UsbDeviceInfo>, Error> 
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

struct DeviceInfoHandler<'a>
{
    device: &'a Device
}
impl<'a> DeviceInfoHandler<'a>
{
    fn new(device: &'a Device) -> Self 
    {
        Self 
        {
            device
        }
    }
}
impl<'a> DeviceInfo<'a> for DeviceInfoHandler<'a>
{
    fn valid_usb_device(&self) -> bool 
    {
        //! Checks if a device is a USB device
        let device_filename = self.device.devpath().to_string_lossy().to_string();
        let device_subsystem = self.device.subsystem().unwrap_or_default();
        device_subsystem.eq_ignore_ascii_case("block") 
        && device_filename.contains("/usb") 
        && self.filesystem().is_some() 
        && self.dev_name().is_some()
    }

    fn vendor(&self) ->  Option<&'a str> 
    {
        self.device
            .property_value("ID_VENDOR")
            .and_then(|s| s.to_str())
    }

    fn description(&self) ->  Option<&'a str> 
    {
        self.device
            .property_value("ID_MODEL_FROM_DATABASE")
            .or(self.device.property_value("ID_MODEL"))
            .and_then(|s| s.to_str())
    }

    fn serial_number(&self) ->  Option<&'a str> 
    {
        self.device
            .property_value("ID_SERIAL_SHORT")
            .or(self.device.property_value("ID_SERIAL"))
            .and_then(|s| s.to_str())
    }

    fn volume_label(&self) ->  Option<&'a str> 
    {
        self.device
            .property_value("ID_FS_LABEL")
            .and_then(|s| s.to_str())
    }

    fn filesystem(&self) ->  Option<&'a str> 
    {
        self.device
            .property_value("ID_FS_TYPE")
            .and_then(|s| s.to_str())
    }

    fn dev_name(&self) ->  Option<&'a str> 
    {
        self.device
            .property_value("DEVNAME")
            .and_then(|s| s.to_str())
    }

    fn fs_id_uuid(&self) ->  Option<&'a str> 
    {
        self.device
            .property_value("ID_FS_UUID")
            .and_then(|s| s.to_str())
    }
    ///mount have litle latency after event
    fn mount_point(&self) ->  Option<PathBuf> 
    {
        let mount_point = if let Some(n) = self.dev_name()
        {
            let path = retry_sync(10, 1000, 2000, ||
            {
                match super::mountpoints::MountPoints::get_mount_point_with_load(n)
                {
                    Ok(r) => Ok(r),
                    Err(e) => Err(e)
                }
            });
            path.ok()
        }
        else
        {
            None
        };
        mount_point
    }
}

impl<'a> Into<UsbDeviceInfo> for DeviceInfoHandler<'a>
{
    fn into(self) -> UsbDeviceInfo 
    {
        UsbDeviceInfo 
        { 
            vendor: self.vendor().and_then(|v| Some(v.to_owned())),
            description: self.description().and_then(|v| Some(v.to_owned())),
            serial_number: self.serial_number().and_then(|v| Some(v.to_owned())),
            volume_label: self.volume_label().and_then(|v| Some(v.to_owned())),
            filesystem: self.filesystem().and_then(|v| Some(v.to_owned())),
            dev_name: self.dev_name().and_then(|v| Some(v.to_owned())),
            fs_id_uuid: self.fs_id_uuid().and_then(|v| Some(v.to_owned())),
            mount_point: self.mount_point()
        }
    }
}


fn is_usb_device(device: &udev::Device) -> bool 
{
    //! Checks if a device is a USB device
    let device_filename = device.devpath().to_string_lossy().to_string();
    let device_subsystem = device.subsystem().unwrap_or_default();
    device_subsystem.eq_ignore_ascii_case("block") && device_filename.contains("/usb")
}
//pub fn on_usb_insert<F>(tasks: Arc<RwLock<HashMap<uuid::Uuid, TaskWithProgress>>>, scheduler: Arc<Scheduler<uuid::Uuid>>, callback: F)
pub fn on_usb_insert<F>(callback: F)
where F: Fn(PathBuf) + Send + 'static
{
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<PathBuf>(1);
    tokio::task::spawn_blocking(move || 
    {
        enumerate(sender);
    });
    tokio::task::spawn_blocking(move ||
    {
        while let Some(r) = receiver.blocking_recv()
        {
            callback(r);
            // r.push(crate::FILE_NAME);
            // if std::fs::exists(&r).is_ok_and(|s| s == true)
            // {

            // }
            // let path_to_config = r.push(crate::FILE_NAME);
            // logger::info!("receive info from receiver! {}", r.display());
        }
    });
}


//pub fn enumerate(tasks: Arc<RwLock<HashMap<uuid::Uuid, TaskWithProgress>>>, scheduler: Arc<Scheduler<uuid::Uuid>>)
fn enumerate(sender: Sender<PathBuf>)
{
    if let Ok(builder) = udev::MonitorBuilder::new()
    {
        if let Ok(matching) = builder.match_subsystem("block")
        {
            if let Ok(socket) = matching.listen()
            {
                let _ = poll(socket, sender);
            }
        }
    }
}
    
fn poll(mut socket: udev::MonitorSocket, sender: Sender<PathBuf>) -> std::io::Result<()> 
{
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    poll.registry().register(
        &mut socket,
        Token(0),
        Interest::READABLE | Interest::WRITABLE,
    )?;

    loop 
    {
        poll.poll(&mut events, None)?;
        for event in &events 
        {
            if event.token() == Token(0) && event.is_writable() 
            {
                for e in socket.iter()
                {
                    let device = e.device();
                    if let EventType::Add =  e.event_type()
                    {
                        let device_handler = DeviceInfoHandler::new(&device);
                        if device_handler.valid_usb_device()
                        {
                            if let Some(mp) = device_handler.mount_point()
                            {
                                let r = sender.blocking_send(mp);
                                logger::debug!("send result: {:?}", r);
                            }
                            //let usb_info: UsbDeviceInfo = device_handler.into();
                            //logger::debug!("{:?}", usb_info);
                        }
                    }
                }
            }
        }
    }
}

//нихрена не понятно но очень интересно....
// pub struct EventsWrapper
// {
//     socket: udev::MonitorSocket,
//     thunk: Option<Pin<Box<dyn Future<Output = Option<UsbDeviceInfo>> + Send>>>
// }
//     impl Stream for EventsWrapper
//     {
//         type Item = UsbDeviceInfo;
//         fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> 
//         {
//             match self.as_mut().socket.iter().next()
//             {
//                 None => std::task::Poll::Ready(None),
//                 Some(event) => 
//                 {
//                     if let Ok(usb) = event.device().try_into()
//                     {
//                         let mut fut = self.thunk.take().unwrap_or_else(move || 
//                         {
//                             Box::pin(async move { Some(usb) })
//                         });
//                             match fut.as_mut().poll(cx) 
//                             {
//                                 std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
//                                 std::task::Poll::Ready(Some(d)) => 
//                                 {
//                                     std::task::Poll::Ready(Some(d))
//                                 },
//                                 std::task::Poll::Pending => 
//                                 {
//                                     // replace the thunk if we're not done with it
//                                     self.thunk = Some(fut);
//                                     std::task::Poll::Pending
//                                 }
//                             }
//                     }
//                     else 
//                     {
//                         std::task::Poll::Ready(None)
//                     }
                    
//                 }
//             }
//         }
//     }


fn print_event(event: udev::Event) 
{
    logger::debug!(
        "{}: {} {} (subsystem={}, sysname={}, devtype={})",
        event.sequence_number(),
        event.event_type(),
        event.syspath().to_str().unwrap_or("---"),
        event
            .subsystem()
            .map_or("", |s| { s.to_str().unwrap_or("") }),
        event.sysname().to_str().unwrap_or(""),
        event.devtype().map_or("", |s| { s.to_str().unwrap_or("") })
    );
}

#[cfg(test)]
mod tests
{
    use std::{collections::HashMap, path::PathBuf, sync::Arc};

    use logger::info;
    use scheduler::Scheduler;
    use tokio::{sync::RwLock, task::spawn_blocking};


    #[tokio::test]
    async fn test_polling()
    {
        logger::StructLogger::new_default();
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<PathBuf>(1);
        tokio::task::spawn_blocking(move ||
        {
            super::enumerate(sender);
        });
        tokio::task::spawn_blocking(move ||
        {
            logger::info!("starting from receiver!");
            while let Some(r) = receiver.blocking_recv()
            {
                logger::info!("receive info from receiver! {}", r.display());
            }
        });
        loop 
        {
            tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
            logger::debug!("looping: 5s");
        }
        
    }

    #[tokio::test]
    async fn on_usb_insert()
    {
        logger::StructLogger::new_default();
        super::on_usb_insert(|c|
        {
            logger::info!("callback fn {}", c.display());
        });
        loop 
        {
            tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
            logger::debug!("looping: 5s");
        }
        
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
    #[test]
    fn test_enumerate()
    {
        logger::StructLogger::new_default();
        let result = super::enumerate_connected_usb().unwrap();
        for device in result 
        {
            info!("{:#?}", device);
        }
    }
}