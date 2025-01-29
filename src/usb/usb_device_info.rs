use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

pub struct Usb{}

