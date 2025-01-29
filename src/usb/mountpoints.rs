use std::{io::BufRead, path::PathBuf};
use super::error::Error;




#[derive(Debug, Clone, Default)]
pub struct Mount 
{
    /// The device name "DEVNAME" "/dev/sdd1"
    pub device: String,
    /// The target directory it has been mounted to
    pub mountpoint: PathBuf,
    /// Type of filesystem
    pub fstype: String,
    /// Mount options
    pub mountopts: String,
}

pub struct MountPoints(Vec<Mount>);
impl MountPoints
{
    ///load information from /proc/mounts
    pub fn load() -> Result<Self, Error> 
    {
        Ok(
            Self(std::io::BufReader::new(
                std::fs::File::open(PathBuf::from("/proc/mounts"))
                    .map_err(|e| Error::ReadFile("/proc/mounts".into(), e))?,
            )
            .lines()
            .map_while(Result::ok)
            .filter_map(|l| 
            {
                let mut parts = l.trim_end_matches(" 0 0").split(' ');
                Some(Mount 
                {
                    device: parts.next()?.into(),
                    mountpoint: parts.next()?.into(),
                    fstype: parts.next()?.into(),
                    mountopts: parts.next()?.into(),
                })
            }).collect())
        )
    }
    pub fn get_mount_point(&self, device_name: &str) -> Option<PathBuf>
    {
        self.0.iter().find(|f| &f.device == device_name).as_ref().and_then(|m| Some(m.mountpoint.clone()))
    }
}

impl Mount
{
    // pub fn list_mount_points() -> Result<impl Iterator<Item = Self>, Error> 
    // {
    //     Ok(
    //         std::io::BufReader::new(
    //             std::fs::File::open(PathBuf::from("/proc/mounts"))
    //                 .map_err(|e| Error::ReadFile("/proc/mounts".into(), e))?,
    //         )
    //         .lines()
    //         .map_while(Result::ok)
    //         .filter_map(|l| 
    //         {
    //             let mut parts = l.trim_end_matches(" 0 0").split(' ');
    //             Some(Self 
    //             {
    //                 device: parts.next()?.into(),
    //                 mountpoint: parts.next()?.into(),
    //                 fstype: parts.next()?.into(),
    //                 mountopts: parts.next()?.into(),
    //             })
    //         })
    //     )
    // }
    // pub fn get_points()
    // {

    // }
    // pub fn get_mount_point(device_name: &str) -> Option<PathBuf>
    // {
    //     let mut mp = Self::list_mount_points().ok()?;
    //     mp.find(|f| &f.device == device_name).as_ref().and_then(|m| Some(m.mountpoint.clone()))
    // }

}