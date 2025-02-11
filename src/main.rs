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
use std::{collections::HashMap, path::Path, sync::Arc, time::Duration};
use futures::{task::SpawnExt, StreamExt};
use indicatif::MultiProgress;
use progressbars::{progress_bar_for_datetime, progress_bar_for_interval};
use scheduler::Scheduler;
use structs::TaskWithProgress;
use config::Config;
use tasker::Handler;
use tokio::{runtime::Handle, sync::RwLock};
use usb::{on_usb_insert, UsbDeviceInfo};



#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() 
{
    let _ = logger::StructLogger::new_default();
    // let (sender, mut receiver) = tokio::sync::mpsc::channel::<UsbDeviceInfo>(1);
    // //TODO проверить этот вариант, возможно не хочет потому что запускалось из futures::ececutor::block_on
    // tokio::task::block_in_place(move ||
    // {
    //     Handle::current().block_on(async move 
    //     {
    //         let r = usb::enumerate_connected_usb(sender).await;
    //         logger::info!("result from enumerate_connected_usb in main: {:?}", r);
    //     }); 
    // });
    // tokio::spawn(async move
    // {
    //     while let Some(stream) = usb_enumerate_receiver.recv().await
    //     {
    //         logger::info!("info from receiver: {:?}", stream);
    //     }
    // });
//    loop 
//    {
//        tokio::time::sleep(Duration::from_millis(5000)).await;
//        logger::debug!("report 5000");
//    }
    let config = if let Ok(config) = Config::load_local()
    {
        config
    }
    else 
    {
        logger::warn!("Локальный файл конфигурации {} не обнаружен, ожидаю ввода...", config::FILE_NAME);
        Config
        {
            tasks: Vec::new()
        }
    };
    run_process(config).await;
}

// async fn load_config()
// {
//     let config = utilites::deserialize::<Config, _>(FILE_NAME, false, utilites::Serializer::Toml);
//     if let Ok(cfg) = config
//     {
//         //del_file(FILE_NAME);
//         println!("Запущен процесс из файла конфигурации");
//         run_process(cfg).await;
//     }
//     else 
//     {
//         println!("Ошибка загрузки файла конфигурации {} {}",FILE_NAME, config.err().as_ref().unwrap());
//         let mut input = String::new();
//         std::io::stdin().read_line(&mut input).expect("Did not enter a correct string");
//     }
// }

async fn run_process(cfg: Config)
{
    let mpb = MultiProgress::default();
    let tasks: Arc<RwLock<HashMap<uuid::Uuid, TaskWithProgress>>> = Arc::new(RwLock::new(HashMap::new()));
    let scheduler: Arc<Scheduler<uuid::Uuid>> = Arc::new(Scheduler::new());
    cfg.add_tasks(mpb.clone(), tasks.clone(), scheduler.clone()).await;
    // for task in cfg.tasks.into_iter()
    // {
    //     let task = TaskWithProgress::new(task, &mpb);
    //     let task_id = uuid::Uuid::new_v4();
    //     let repeating = *task.get_strategy();
    //     if task.path_is_exists()
    //     {
    //         if let Some(i) = task.get_interval()
    //         {
    //             let _ = scheduler.add_interval_task(task_id, i, repeating).await;
    //         }
    //         else if let Some(d) = task.get_date()
    //         {
    //             let _ = scheduler.add_date_task(task_id, d, repeating).await;
    //         }
    //     }
    //     else 
    //     {
    //         let _ = mpb.println(format!("файл не найден {}", task.get_str_path()));
    //     }
    //     let mut guard = tasks.write().await;
    //     guard.insert(task_id, task);
    // }
    let tasks_cl = tasks.clone();
    let scheduler_cl = scheduler.clone();
    on_usb_insert(move |c|
    {
        let path = Path::new(&c).join(config::FILE_NAME);
        logger::debug!("usb path: {}", path.display());
        if let Ok(cfg) = Config::load_from_path(path)
        {
            let mpb_cl = mpb.clone();
            let tasks_async = tasks_cl.clone();
            let scheduler_async = scheduler_cl.clone();
            tokio::spawn(async move 
            {
                cfg.add_tasks(mpb_cl, tasks_async, scheduler_async).await
            });
            
        }
        //logger::info!("callback pathbuf {}", c.display());
    });
    //TODO insert usb handler here
    let handler = Handler::new(tasks);
    //#[cfg(feature = "window")]
    //window::start();
    scheduler.run(handler).await;
}


#[cfg(test)]
mod tests
{
    use std::path::PathBuf;
    use scheduler::RepeatingStrategy;
    use utilites::Date;
    use crate::{helpers::time_diff, structs::{Task}, config::{FILE_NAME, Config}};

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
        let _ = logger::StructLogger::new_default();
        let _ = std::fs::File::create_new("/hard/xar/projects/tests/1");
        let _ = std::fs::File::create_new("/hard/xar/projects/tests/2");
        let _ = std::fs::File::create_new("/hard/xar/projects/tests/3");
        let _ = std::fs::File::create_new("/hard/xar/projects/tests/4");
        let _ = std::fs::File::create_new("/hard/xar/projects/tests/expired");
        let _ = std::fs::create_dir("/hard/xar/projects/tests/5");
        
        let cfg = Config
        {
                tasks: vec![
                Task
                {
                    path: PathBuf::from("/hard/xar/projects/tests/1"),
                    mask: None,
                    interval: Some(2),
                    date: None,
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/hard/xar/projects/tests/2"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/hard/xar/projects/tests/3"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(6)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/hard/xar/projects/tests/4"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Dialy,
                    visible: false
                },
                Task
                {
                    path: PathBuf::from("/hard/xar/projects/tests/5"),
                    mask: Some("*.test".into()),
                    interval: Some(1),
                    date: None,
                    repeat: RepeatingStrategy::Monthly,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/hard/xar/projects/tests/not_exists"),
                    mask: None,
                    interval: Some(1),
                    date: None,
                    repeat: RepeatingStrategy::Monthly,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/hard/xar/projects/tests/expired"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().sub_minutes(3)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
            ]
        };
        let r = utilites::serialize(cfg, FILE_NAME, false, utilites::Serializer::Toml);
        //super::main();
        logger::info!("{:?}", r)
    }

    #[test]
    fn test_serialize_noute()
    {
        let _ = logger::StructLogger::new_default();
        let _ = std::fs::File::create_new("/home/phobos/projects/rust/deltime/tests/1");
        let _ = std::fs::File::create_new("/home/phobos/projects/rust/deltime/tests/2");
        let _ = std::fs::File::create_new("/home/phobos/projects/rust/deltime/tests/3");
        let _ = std::fs::File::create_new("/home/phobos/projects/rust/deltime/tests/4");
        let _ = std::fs::File::create_new("/home/phobos/projects/rust/deltime/tests/expired");
        let _ = std::fs::create_dir("/home/phobos/projects/rust/deltime/tests/5");
        
        let cfg = Config
        {
                tasks: vec![
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/1"),
                    mask: None,
                    interval: Some(2),
                    date: None,
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/2"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/3"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(6)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/4"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Dialy,
                    visible: false
                },
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/5"),
                    mask: Some("*.test".into()),
                    interval: Some(1),
                    date: None,
                    repeat: RepeatingStrategy::Monthly,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/not_exists"),
                    mask: None,
                    interval: Some(1),
                    date: None,
                    repeat: RepeatingStrategy::Monthly,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/expired"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().sub_minutes(3)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
            ]
        };
        let r = utilites::serialize(cfg, FILE_NAME, false, utilites::Serializer::Toml);
        let _ = std::fs::File::create_new("/home/phobos/projects/rust/deltime/tests/usb_1");
        let _ = std::fs::File::create_new("/home/phobos/projects/rust/deltime/tests/usb_2");
        let _ = std::fs::File::create_new("/home/phobos/projects/rust/deltime/tests/usb_3");
        let _ = std::fs::File::create_new("/home/phobos/projects/rust/deltime/tests/usb_4");
        let cfg = Config
        {
                tasks: vec![
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/usb_1"),
                    mask: None,
                    interval: Some(2),
                    date: None,
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/usb_2"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/usb_3"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(6)),
                    repeat: RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    path: PathBuf::from("/home/phobos/projects/rust/deltime/tests/usb_4"),
                    mask: None,
                    interval: None,
                    date: Some(Date::now().add_minutes(3)),
                    repeat: RepeatingStrategy::Dialy,
                    visible: false
                },
            ]
        };
        let r = utilites::serialize(cfg, "usb_config.toml", false, utilites::Serializer::Toml);
        //super::main();
        logger::info!("{:?}", r)
    }

    // #[tokio::test]
    // async fn test_tasker()
    // {
    //     logger::StructLogger::new_default();
    //     let interval_without_repeat = Task
    //     {
    //         path: PathBuf::from("/hard/xar/projects/tests/1"),
    //         mask: None,
    //         interval: Some(1),
    //         date: None,
    //         repeat: RepeatingStrategy::Once,
    //         visible: true
    //     };

    //     let date_task = Task
    //     {
    //         path: PathBuf::from("/hard/xar/projects/tests/2"),
    //         mask: None,
    //         interval: None,
    //         date: Some(Date::now().add_minutes(2)),
    //         repeat: RepeatingStrategy::Once,
    //         visible: true
    //     };

    //     let interval_with_repeat = Task
    //     {
    //         path: PathBuf::from("/hard/xar/projects/tests/3"),
    //         mask: None,
    //         interval: Some(3),
    //         date: None,
    //         repeat: RepeatingStrategy::Once,
    //         visible: true
    //     };

    //     let dialy_date = Task
    //     {
    //         path: PathBuf::from("/hard/xar/projects/tests/4"),
    //         mask: None,
    //         interval: None,
    //         date: Some(Date::now().add_minutes(4)),
    //         repeat: RepeatingStrategy::Once,
    //         visible: true
    //     };

    //     let monthly_date = Task
    //     {
    //         path: PathBuf::from("/hard/xar/projects/tests/5"),
    //         mask: None,
    //         interval: None,
    //         date: Some(Date::now().add_minutes(5)),
    //         repeat: RepeatingStrategy::Once,
    //         visible: true
    //     };

    //     let passed_date = Task
    //     {
    //         path: PathBuf::from("/hard/xar/projects/tests/6"),
    //         mask: None,
    //         interval: None,
    //         date: Some(Date::now().sub_minutes(5)),
    //         repeat: RepeatingStrategy::Once,
    //         visible: true
    //     };

    //     let tasker = Scheduler::new();
    //     let _ = tasker.add_interval_task(Arc::new(interval_without_repeat), 1, RepeatingStrategy::Once).await;
    //     let _ = tasker.add_date_task(Arc::new(date_task), Date::now().add_minutes(2), RepeatingStrategy::Once).await;
    //     let _ = tasker.add_interval_task(Arc::new(interval_with_repeat), 3, RepeatingStrategy::Once).await;
    //     let _ = tasker.add_date_task(Arc::new(dialy_date), Date::now().add_minutes(4), RepeatingStrategy::Dialy).await;
    //     let _ = tasker.add_date_task(Arc::new(monthly_date), Date::now().add_minutes(5), RepeatingStrategy::Monthly).await;
    //     let _ = tasker.add_date_task(Arc::new(passed_date), Date::now().sub_minutes(5), RepeatingStrategy::Monthly).await;
    //     //let (sender, mut receiver) = tokio::sync::mpsc::channel::<Option<(Arc<Task>, Option<Date>)>>(10);
    //     let (sender, mut receiver) = tasker.get_channel();
    //     tokio::spawn(async move
    //     {
    //         while let Some(r) = receiver.recv().await
    //         {
    //             logger::info!("задание:{:?}", r);
    //         }
    //     });
    //     logger::info!("start tasker");
    //     tasker.run(sender).await;
    //     loop 
    //     {
    //         tokio::time::sleep(tokio::time::Duration::from_millis(6000)).await;
    //         logger::info!("loop");
    //     }
    // }
}
