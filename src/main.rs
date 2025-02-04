mod cli;
mod structs;
mod progressbars;
mod usb;
mod tasker;
use std::{sync::{Arc, LazyLock}, thread, time::Duration};
use async_channel::Receiver;
use cli::Cli;
use indicatif::MultiProgress;
use progressbars::{progress_bar_for_datetime, progress_bar_for_interval, set_date_message};
use structs::Config;
use tasker::{ProcessStatus, RepeatingStrategy, Tasker};
use usb::UsbDeviceInfo;
use utilites::Date;
use tokio::{runtime::Handle, sync::{mpsc::channel, RwLock}};
//use async_channel::unbounded;
//use tokio::sync::mpsc::channel;
const FILE_NAME: &str = "config.toml";
static TASKS: LazyLock<Arc<RwLock<Vec<(&structs::Task, indicatif::ProgressBar)>>>> = LazyLock::new(|| Arc::new(RwLock::new(Vec::new())));
#[tokio::main]
async fn main() 
{
    logger::StructLogger::new_default();
    // let (sender, mut receiver) = channel::<UsbDeviceInfo>(5);
    // //usb::enumerate_connected_usb(sender).await;
    // let handle = Handle::current();
    // tokio::task::block_in_place(move ||
    // {
    //     logger::info!("block_in_place");
    //     //handle.block_on(async move {usb::enumerate_connected_usb(sender.clone()).await})
        
    // });
    // logger::info!("after block_in_place");
    // tokio::spawn(async move
    // {
    //     while let Some(s) = receiver.recv().await
    //     {
    //         logger::info!("{:?}", s);
    //     }
    // });
    // loop 
    // {
    //     tokio::time::sleep(tokio::time::Duration::from_millis(6000)).await;
    //     logger::debug!("тест")

    // }
   
    load_config().await;
}

async fn load_config()
{
    
    let config = utilites::deserialize::<Config, _>(FILE_NAME, false, utilites::Serializer::Toml);
    if let Ok(cfg) = config
    {
        //del_file(FILE_NAME);
        println!("Запущен процесс из файла конфигурации");
        run_process(cfg).await;
    }
    else 
    {
        println!("Ошибка загрузки файла конфигурации {} {}",FILE_NAME, config.err().as_ref().unwrap());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Did not enter a correct string");
    }
}

async fn run_process(cfg: Config)
{
    let mpb = MultiProgress::default();
    let tasker = Tasker::new();
    let (sender, mut receiver) = tasker.get_channel();
    for task in cfg.tasks.into_iter()
    {
        if std::fs::exists(&task.file_path).is_ok_and(|f| f == true)
        {
            let repeating = task.repeat;
            let path = task.file_path.clone();
            if let Some(i) = task.del_time_interval
            {
                
                let pb = progress_bar_for_interval(&mpb, &repeating, task.visible, &path, i);
                if let RepeatingStrategy::Forever | RepeatingStrategy::Dialy | RepeatingStrategy::Monthly = &repeating
                {
                    pb.set_prefix("♾ ");
                }
                let _ = tasker.add_interval_task((task, pb), i, repeating).await;
            }
            else if let Some(d) = task.del_time.clone()
            {
                let now = Date::now();
                let target = time_diff(&now, &d);
                let pb = progress_bar_for_datetime(&mpb, task.visible, &path, &d, target as u32);
                if let RepeatingStrategy::Forever | RepeatingStrategy::Dialy | RepeatingStrategy::Monthly = &repeating
                {
                    pb.set_prefix("♾ ");
                }
                let _ = tasker.add_date_task((task, pb), &d, repeating).await;
            }
        }
        else 
        {
            
            let path = task.file_path.clone();
            let pb = progress_bar_for_interval(&mpb, &RepeatingStrategy::Once, true, &path, 0);
            pb.set_prefix("❌");
           
            //pb.println("Ошибка, файл не существует, задача выполнена не будет");
            let _ = tasker.add_interval_task((task, pb), 999, RepeatingStrategy::Once).await;
            //println!("Ошибка, файл {} не существует, задача выполнена не будет", &task.file_path);
        }
    }
    tokio::spawn(async move
    {
        while let Some(r) = receiver.recv().await
        {
            match r
            {
                ProcessStatus::Finish(t) =>
                {
                    let guard = t.read().await;
                    guard.1.set_prefix("✅");
                    guard.1.finish();
                },
                ProcessStatus::Tick(t) =>
                {
                    let guard = t.read().await;
                    if let Some(date) = guard.0.del_time.as_ref()
                    {
                        let current_date = Date::now();
                        let diff = time_diff(&current_date, date);
                        if diff.is_positive()
                        {
                            guard.1.set_position(guard.1.length().unwrap() - diff as u64);  
                        }
                    }
                    else if let Some(_) = guard.0.del_time_interval.as_ref()
                    {
                        guard.1.inc(1);
                    }
                },
                ProcessStatus::FinishCycle(t) =>
                {
                    let date = t.1;
                    let mut guard = t.0.write().await;
                    if guard.0.del_time.is_some()
                    {
                        if date.is_some()
                        {
                            let date_link = date.as_ref().unwrap();
                            guard.1.reset(); 
                            let current_date = Date::now();
                            let new_len = time_diff(&current_date, date_link);
                            guard.1.set_length(new_len as u64);
                            set_date_message(&guard.1, guard.0.visible, date_link, &guard.0.file_path);
                            guard.0.del_time = date;
                            
                        }
                    }
                    else 
                    {
                        guard.1.reset(); 
                    }
                    
                }
            }
        }
    });
    tasker.run(sender).await;
}

// fn run_process(cfg: Config)
// {

//     let tasks = Arc::new(RwLock::new(Vec::with_capacity(cfg.tasks.len())));
//     for t in cfg.tasks.iter()
//     {
//         if std::fs::exists(&t.file_path).is_ok_and(|f| f == true)
//         {
//             if let Some(i) = t.del_time_interval.as_ref()
//             {
//                 let pb = progress_bar_for_interval(&mpb, t.repeat, t.visible, &t.file_path, *i);
//                 let mut guard = tasks.write().unwrap();
//                 guard.push((t, pb)); 
//             }
//             if let Some(d) = t.del_time.as_ref()
//             {
//                 let now = Date::now();
//                 let target = time_diff(&now, d);
//                 let pb = progress_bar_for_datetime(&mpb, t.visible, &t.file_path, d, target as u32);
//                 let mut guard = tasks.write().unwrap();
//                 guard.push((t, pb)); 
//             }
//         }
//         else 
//         {
//             println!("Ошибка, файл {} не существует, задача выполнена не будет", &t.file_path);
//         }
//     }
    
// }


// pub fn process()
// {
//     let mpb = MultiProgress::default();
//     let mut minutes: u32 = 0;
//     loop
//     {
//         let tasks = TASKS.read().unwrap();
//         let mut del_tasks = Vec::new();
//         for t in tasks.iter()
//         {
//             if let Some(interval) = t.0.del_time_interval.as_ref()
//             {
//                 if minutes != 0
//                 {
//                     t.1.inc(1);
//                     if  minutes % interval == 0
//                     {
//                         if t.0.repeat
//                         {
//                             t.1.reset();
//                         }
//                         else 
//                         {
//                             t.1.finish();
//                         }
//                         if del_file(&t.0.file_path) && !t.0.repeat
//                         {
//                             del_tasks.push(t.0.file_path.clone());
//                             t.1.set_prefix("✅");
//                         }
//                         else 
//                         {
//                             t.1.set_prefix("❌");
//                         }
//                     }
//                 }
//             }
//             else if let Some(checked_date) = t.0.del_time.as_ref()
//             {
//                 let current_date = Date::now();
//                 let diff = time_diff(&current_date, checked_date);
//                 if diff.is_negative()
//                 {
//                     if del_file(&t.0.file_path)
//                     {
//                         t.1.set_prefix("✅");
//                         del_tasks.push(t.0.file_path.clone());
//                         t.1.finish();
//                     }
//                     else 
//                     {
//                         t.1.set_prefix("❌");
//                     }
//                 }
//                 else 
//                 {
//                     t.1.set_position(t.1.length().unwrap() - diff as u64);  
//                 }
//             }
//         }
//         drop(tasks);
//         let mut tasks = TASKS.write().unwrap();
//         for d in del_tasks
//         {
//             tasks.retain(|r| r.0.file_path != d);
//         }
//         drop(tasks);
//         std::thread::sleep(Duration::from_secs(60));
//         minutes += 1;
//     }
// }



///Получаем отрицательное значение если проверяемая дата меньше текущей
fn time_diff(current_date: &Date, checked_date: &Date) -> i64
{
    checked_date.as_naive_datetime().and_utc().timestamp() - current_date.as_naive_datetime().and_utc().timestamp()
}

fn del_file(path: &str) -> bool
{
    let metadata = std::fs::metadata(path);
    if let Ok(md) = metadata
    {
        if md.is_file()
        {
            let del = std::fs::remove_file(path);
            return if del.is_ok()
            {
                true
            }
            else 
            {
                false
            };
        }
        if md.is_dir()
        {
            let del = std::fs::remove_dir_all(path);
            return if del.is_ok()
            {
                true
            }
            else 
            {
                false
            };
        }
    }
    return false;
    
}


#[cfg(test)]
mod tests
{
    use std::{os::raw, sync::Arc};

    use utilites::Date;

    use crate::{structs::{Config, Task}, tasker::Tasker, time_diff, FILE_NAME};

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
        let _ = std::fs::create_dir("/hard/xar/projects/tests/5");
        let cfg = Config
        {
                tasks: vec![
                Task
                {
                    file_path: "/hard/xar/projects/tests/1".to_owned(),
                    del_time_interval: Some(2),
                    del_time: None,
                    repeat: crate::tasker::RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    file_path: "/hard/xar/projects/tests/2".to_owned(),
                    del_time_interval: None,
                    del_time: Some(Date::now().add_minutes(3)),
                    repeat: crate::tasker::RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    file_path: "/hard/xar/projects/tests/3".to_owned(),
                    del_time_interval: None,
                    del_time: Some(Date::now().add_minutes(6)),
                    repeat: crate::tasker::RepeatingStrategy::Once,
                    visible: true
                },
                Task
                {
                    file_path: "/hard/xar/projects/tests/4".to_owned(),
                    del_time_interval: Some(2),
                    del_time: None,
                    repeat: crate::tasker::RepeatingStrategy::Dialy,
                    visible: false
                },
                Task
                {
                    file_path: "/hard/xar/projects/tests/5".to_owned(),
                    del_time_interval: Some(1),
                    del_time: None,
                    repeat: crate::tasker::RepeatingStrategy::Monthly,
                    visible: true
                },
                Task
                {
                    file_path: "/hard/xar/projects/tests/not_exists".to_owned(),
                    del_time_interval: Some(1),
                    del_time: None,
                    repeat: crate::tasker::RepeatingStrategy::Monthly,
                    visible: true
                },
            ]
        };
        let r = utilites::serialize(cfg, FILE_NAME, false, utilites::Serializer::Toml);
        //super::main();
        logger::info!("{:?}", r)
    }

    #[tokio::test]
    async fn test_tasker()
    {
        logger::StructLogger::new_default();
        let interval_without_repeat = Task
        {
            file_path: "/hard/xar/projects/tests/1".to_owned(),
            del_time_interval: Some(1),
            del_time: None,
            repeat: crate::tasker::RepeatingStrategy::Once,
            visible: true
        };
        let date_task = Task
        {
            file_path: "/hard/xar/projects/tests/2".to_owned(),
            del_time_interval: None,
            del_time: Some(Date::now().add_minutes(2)),
            repeat: crate::tasker::RepeatingStrategy::Once,
            visible: true
        };
        let interval_with_repeat = Task
        {
            file_path: "/hard/xar/projects/tests/3".to_owned(),
            del_time_interval: Some(3),
            del_time: None,
            repeat: crate::tasker::RepeatingStrategy::Once,
            visible: true
        };
        let dialy_date = Task
        {
            file_path: "/hard/xar/projects/tests/4".to_owned(),
            del_time_interval: None,
            del_time: Some(Date::now().add_minutes(4)),
            repeat: crate::tasker::RepeatingStrategy::Once,
            visible: true
        };
        let monthly_date = Task
        {
            file_path: "/hard/xar/projects/tests/5".to_owned(),
            del_time_interval: None,
            del_time: Some(Date::now().add_minutes(5)),
            repeat: crate::tasker::RepeatingStrategy::Once,
            visible: true
        };

        let passed_date = Task
        {
            file_path: "/hard/xar/projects/tests/6".to_owned(),
            del_time_interval: None,
            del_time: Some(Date::now().sub_minutes(5)),
            repeat: crate::tasker::RepeatingStrategy::Once,
            visible: true
        };


        let tasker = Tasker::new();
        let _ = tasker.add_interval_task(Arc::new(interval_without_repeat), 1, crate::tasker::RepeatingStrategy::Once).await;
        let _ = tasker.add_date_task(Arc::new(date_task), &Date::now().add_minutes(2), crate::tasker::RepeatingStrategy::Once).await;
        let _ = tasker.add_interval_task(Arc::new(interval_with_repeat), 3, crate::tasker::RepeatingStrategy::Once).await;
        let _ = tasker.add_date_task(Arc::new(dialy_date), &Date::now().add_minutes(4), crate::tasker::RepeatingStrategy::Dialy).await;
        let _ = tasker.add_date_task(Arc::new(monthly_date), &Date::now().add_minutes(5), crate::tasker::RepeatingStrategy::Monthly).await;
        let _ = tasker.add_date_task(Arc::new(passed_date), &Date::now().sub_minutes(5), crate::tasker::RepeatingStrategy::Monthly).await;
        //let (sender, mut receiver) = tokio::sync::mpsc::channel::<Option<(Arc<Task>, Option<Date>)>>(10);
        let (sender, mut receiver) = tasker.get_channel();
        tokio::spawn(async move
        {
            while let Some(r) = receiver.recv().await
            {
                
                logger::info!("задание:{:?}", r);
            }
        });
        logger::info!("start tasker");
        tasker.run(sender).await;
        loop 
        {
            tokio::time::sleep(tokio::time::Duration::from_millis(6000)).await;
            logger::info!("loop");
        }
    }
}
