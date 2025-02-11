use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub trait DeviceInfo<'a>
{
    fn valid_usb_device(&self) -> bool;
    fn vendor(&self) ->  Option<&'a str>;
    fn description(&self) ->  Option<&'a str>;
    fn serial_number(&self) ->  Option<&'a str>;
    fn volume_label(&self) ->  Option<&'a str>;
    fn filesystem(&self) ->  Option<&'a str>;
    fn dev_name(&self) ->  Option<&'a str>;
    fn fs_id_uuid(&self) ->  Option<&'a str>;
    fn mount_point(&self) ->  Option<PathBuf>;
}

#[derive(PartialEq, Hash, Clone, Debug, Default, Deserialize, Serialize)]
pub struct UsbDeviceInfo
{
    pub vendor: Option<String>,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub volume_label: Option<String>,
    pub filesystem: Option<String>,
    pub dev_name: Option<String>,
    pub fs_id_uuid: Option<String>,
    pub mount_point: Option<PathBuf>
}

