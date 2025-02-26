use std::path::PathBuf;
use crate::{error::Error, helpers::ReceiverStream};
use super::usb_device_info::{DeviceInfo, UsbDeviceInfo};
use futures::Stream;
use udev::{mio::{Events, Interest, Poll, Token}, Device, EventType, MonitorSocket, MonitorSocketIter};
use utilites::retry_sync;


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


pub fn usb_event() -> Result<impl Stream<Item = PathBuf>, Error> 
{
    let (sender, receiver) = tokio::sync::mpsc::channel::<PathBuf>(1);
    std::thread::spawn(move ||
    {
        //closure for error handing
        //udev::MonitorSocket not thread safe 
        let r = || 
        {
            let mut polling = Polling::new()?.register()?;
            loop 
            {
                let events = polling.check()?;
                if let Some(event_iter) = events
                {
                    for e in event_iter 
                    {
                        let device = e.device();
                        if let EventType::Add =  e.event_type()
                        {
                            let device_handler = DeviceInfoHandler::new(&device);
                            if device_handler.valid_usb_device()
                            {
                                if let Some(mp) = device_handler.mount_point()
                                {
                                    logger::debug!("обнаружена точка монтирования usb flash накопителя: {}", mp.display());
                                    let _ = sender.blocking_send(mp);
                                }
                            }
                        }
                    }
                }
            }
            #[allow(unreachable_code)]
            //use for correct type inference in closure
            return Ok(());
        };
        let res: Result<(), Error> = r();
        if res.is_err()
        {
            logger::error!("{:?}", res.err().unwrap());
        }
    });
    Ok(ReceiverStream::new(receiver))
}

struct Polling
{
    poll: Poll,
    socket: udev::MonitorSocket,
    events: Events
}

impl Polling
{
    pub fn new() -> Result<Self, Error>
    {
        let poll = Poll::new()?;
        let socket = Self::get_socket();
        if socket.is_none()
        {
            return Err(Error::Generic("Ошибка сокета".into()));
        }
        let events = Events::with_capacity(1024);
        let socket = socket.unwrap();
        Ok(Self
        {
            poll,
            socket,
            events
        })
    }

    pub fn register(mut self) -> Result<Self, Error>
    {
        self.poll.registry().register(
            &mut self.socket,
            Token(0),
            Interest::READABLE | Interest::WRITABLE,
        )?;
        Ok(self)
    }

    pub fn check<'e>(&'e mut self) -> Result<Option<MonitorSocketIter<'e>>, Error>
    {
        //let mut events = Events::with_capacity(1024);
        self.poll.poll(&mut self.events, None)?;
        if self.events.iter().any(|a| a.token() == Token(0) && a.is_writable())
        {
            Ok(Some(self.socket.iter()))
        }
        else 
        {
            Ok(None)
        }
    }

    fn get_socket() -> Option<MonitorSocket>
    {
        if let Ok(builder) = udev::MonitorBuilder::new()
        {
            if let Ok(matching) = builder.match_subsystem("block")
            {
                if let Ok(socket) = matching.listen()
                {
                    return Some(socket);
                }
            }
        }
        None
    }

}

#[cfg(test)]
mod tests
{
    use futures::StreamExt;
    use logger::info;

    #[tokio::test]
    async fn on_usb_insert()
    {
        let _ = logger::StructLogger::new_default();
        if let Ok(poll) = super::usb_event().as_mut()
        {
            while let Some(p) = poll.next().await
            {
                logger::info!("{}",p.display())
            }
        }
    }
    
    #[test]
    fn test_udev()
    {
        let _ = logger::StructLogger::new_default();
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