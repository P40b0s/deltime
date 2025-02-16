use std::{io::BufRead, path::PathBuf};
use crate::error::Error;



#[derive(Debug, Clone, Default)]
pub struct Mount 
{
    /// The device name "DEVNAME" "/dev/sdd1"
    pub device: String,
    /// The target directory it has been mounted to
    pub mountpoint: PathBuf,
    #[allow(dead_code)]
    /// Type of filesystem
    pub fstype: String,
    #[allow(dead_code)]
    /// Mount options
    pub mountopts: String,
}

pub struct MountPoints(Vec<Mount>);
impl MountPoints
{
    ///load information from /proc/mounts
    #[allow(dead_code)]
    pub fn load() -> Result<Self, Error> 
    {
        Ok(
            Self(std::io::BufReader::new(
                std::fs::File::open(PathBuf::from("/proc/mounts"))
                    .map_err(|_| Error::Generic("error accsess /proc/mounts".into()))?,
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
    pub fn get_mount_point_with_load(device_name: &str) -> Result<PathBuf, Error>
    {
        let mnt = std::io::BufReader::new(
            std::fs::File::open(PathBuf::from("/proc/mounts"))
                .map_err(|_| Error::Generic("error access /proc/mounts".into()))?,
        )
        .lines()
        .map_while(Result::ok)
        .find_map(|l| 
        {
            let mut parts = l.trim_end_matches(" 0 0").split(' ');
            let dev_name = parts.next()?;
            let mnt_point = parts.next()?;
            if dev_name == device_name
            {
                Some(mnt_point.to_owned())
            }
            else 
            {
                None
            }
        });
        if let Some(mnt) = mnt
        {
            Ok(mnt.into())
        }
        else 
        {
            Err(Error::Generic("mount point not found".into()))
        }
    }
    #[allow(dead_code)]
    pub fn get_mount_point(&self, device_name: &str) -> Option<PathBuf>
    {
        self.0.iter().find(|f| &f.device == device_name).as_ref().and_then(|m| Some(m.mountpoint.clone()))
    }
}