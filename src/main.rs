//hide process when start
#![windows_subsystem = "windows"]
mod structs;
mod progressbars;
mod usb;
mod error;
mod tasker;
mod helpers;
#[cfg(feature = "window")]
mod window;
mod config;
#[cfg(feature = "beeper")]
mod beeper;
use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc};
use futures::StreamExt;
use indicatif::MultiProgress;
use progressbars::{progress_bar_for_datetime, progress_bar_for_interval};
use scheduler::Scheduler;
use structs::TaskWithProgress;
use config::Config;
use tasker::Handler;
use tokio::sync::RwLock;
use usb::usb_event;



#[tokio::main]
async fn main() 
{
    let _ = logger::StructLogger::new_default();
    let config =  Config::load().await;
    run_process(config).await;
}

async fn run_process(cfg: Config)
{
    let mpb = MultiProgress::default();
    let tasks: Arc<RwLock<HashMap<Arc<String>, TaskWithProgress>>> = Arc::new(RwLock::new(HashMap::new()));
    let scheduler: Scheduler<Arc<String>> = Scheduler::new();
    cfg.add_tasks(mpb.clone(), tasks.clone(), scheduler.clone()).await;
    usb_checker(mpb.clone(), tasks.clone(), scheduler.clone());
    let handler = Handler::new(tasks);
    //hide process when start
    #[cfg(all(target_os = "linux", feature = "window"))]
    window::start();
    scheduler.run(handler).await;
}

#[cfg(all(target_os = "linux", feature = "usb"))]
fn usb_checker(mpb: MultiProgress, tasks:  Arc<RwLock<HashMap<Arc<String>, TaskWithProgress>>>, scheduler: Scheduler<Arc<String>>)
{   
    tokio::spawn(async move 
    {
        if let Ok(stream) = usb_event().as_mut()
        {
            while let Some(path) = stream.next().await
            {
                usb_path_worker(mpb.clone(), tasks.clone(), scheduler.clone(), path).await;
            }
        }
    });
}
///correctly working if wrapping into futures executor
#[cfg(all(target_os = "windows", feature = "usb"))]
fn usb_checker(mpb: MultiProgress, tasks:  Arc<RwLock<HashMap<Arc<String>, TaskWithProgress>>>, scheduler: Scheduler<Arc<String>>)
{   
    tokio::task::spawn_blocking(move ||
    {
        futures::executor::block_on(async 
        {
            if let Ok(stream) = usb_event().as_mut()
            {
                while let Some(path) = stream.next().await
                {
                    usb_path_worker(mpb.clone(), tasks.clone(), scheduler.clone(), path).await;
                }
            }
        });
    });
    
}

async fn usb_path_worker(mpb: MultiProgress, tasks:  Arc<RwLock<HashMap<Arc<String>, TaskWithProgress>>>, scheduler: Scheduler<Arc<String>>, path: PathBuf)
{
    let path = Path::new(&path).join(config::FILE_NAME);
    //logger::debug!("usb path: {}", path.display());
    let config = Config::load_from_path(&path);
    if let Ok(cfg) = config
    {
        let _ = mpb.println(format!("Файл конфигурации успешно загружен с найденого накопителя {}", path.display()));
        cfg.add_tasks(mpb, tasks, scheduler).await
    }
    else 
    {
        logger::error!("Отсутсвует файл {} -> {}", path.display(), config.err().unwrap());
    }
}


#[cfg(test)]
mod tests
{
    use std::path::PathBuf;
    use scheduler::RepeatingStrategy;
    use utilites::Date;
    use crate::{helpers::time_diff, structs::Task, config::{FILE_NAME, Config}};

    #[test]
    fn test_deserialize()
    {
        let _ = logger::StructLogger::new_default();
        let config = utilites::deserialize::<Config, _>(FILE_NAME, false, utilites::Serializer::Toml);
        logger::info!("{:?}", config);
    }

    #[test]
    fn test_date_time_diff()
    {
        let _ = logger::StructLogger::new_default();
        let d1 = Date::parse("2022-10-26T13:23:52").unwrap();
        let d2 = Date::parse("2022-10-26T13:24:52").unwrap();
        let diff = time_diff(&d1, &d2);
        logger::info!("{:?}", diff);
    }

    #[test]
    fn test_serialize()
    {
        let path = "/home/phobos/projects/rust/deltime/tests/";
        let flash = "/run/media/phobos/x/";
        //let path = "/hard/xar/projects/rust/deltime/tests/";
        let name = |n: &str|
        {
            [path, n].concat()
        };
        let _ = logger::StructLogger::new_default();
        let _ = std::fs::File::create_new(name("1"));
        let _ = std::fs::File::create_new(name("2"));
        let _ = std::fs::File::create_new(name("3"));
        let _ = std::fs::File::create_new(name("4"));
        let _ = std::fs::File::create_new(name("expired"));
        let _ = std::fs::create_dir(name("5"));
        let _ = std::fs::File::create_new(name("5/delme_by_extension.delme"));
        let _ = std::fs::File::create_new(name("5/not_delme.test"));
        
        let cfg = Config
        {
                tasks: vec![
                Task
                {
                    path: PathBuf::from(name("1")),
                    mask: None,
                    interval: Some(1),
                    date: None,
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from(name("2")),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from(name("3")),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(6)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from(name("4")),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Dialy,
                    visible: false
                },
                Task
                {
                    path: PathBuf::from(name("5")),
                    mask: Some("*.delme".into()),
                    interval: Some(3),
                    date: None,
                    repeat: RepeatingStrategy::Forever,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from(name("not_exists")),
                    mask: None,
                    interval: Some(1),
                    date: None,
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from(name("expired")),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().sub_minutes(3)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
            ]
        };
        let _ = utilites::serialize(cfg, FILE_NAME, false, utilites::Serializer::Toml);
        //usb test
        let _ = std::fs::File::create_new(name("usb_1"));
        let _ = std::fs::File::create_new(name("usb_2"));
        let _ = std::fs::File::create_new(name("usb_3"));
        let _ = std::fs::File::create_new(name("usb_4"));
        let cfg = Config
        {
                tasks: vec![
                Task
                {
                    path: PathBuf::from(name("usb_1")),
                    mask: None,
                    interval: Some(2),
                    date: None,
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from(name("usb_2")),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from(name("usb_3")),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(6)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from(name("usb_4")),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Dialy,
                    visible: false
                },
            ]
        };
        let r = utilites::serialize(cfg, [flash, "config.toml"].concat(), false, utilites::Serializer::Toml);
        //super::main();
        logger::info!("{:?}", r)
    }
}
