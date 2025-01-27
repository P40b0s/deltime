mod cli;
mod structs;
use std::time::Duration;
use cli::Cli;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use structs::Config;
use utilites::Date;
const FILE_NAME: &str = "config.toml";


fn main() 
{
    load_config();
}

fn load_config()
{
    if let Some(args_config) = Cli::parse_args()
    {
        let cfg: Result<Config, String> = args_config.try_into();
        if let Ok(r) = cfg
        {
            println!("Запущен процесс из аргуменов командной строки");
            run_process(r);
        }
        else 
        {
            println!("{}", cfg.err().unwrap());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Did not enter a correct string");
        }
    }
    else
    {
        println!("Аргументы не переданы, или переданы неправильно, попытка запуска процесса из файла конфигурации");
        let config = utilites::deserialize::<Config, _>(FILE_NAME, false, utilites::Serializer::Toml);
        if let Ok(cfg) = config
        {
            del_file(FILE_NAME);
            println!("Запущен процесс из файла конфигурации");
            run_process(cfg);
        }
        else 
        {
            println!("Ошибка загрузки файла конфигурации {} {}",FILE_NAME, config.err().as_ref().unwrap());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Did not enter a correct string");
        }
    }

}

fn progress_bar_for_interval(mpb: &MultiProgress, repeating: bool, visible: bool, fp: &str, len: u32) -> ProgressBar
{
    let pb = mpb.add(ProgressBar::new(len as u64));
    let msg= if visible
    {
        [" -> ", fp].concat()
    }
    else
    {
        "".to_owned()
    };
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(msg);
    let sty = if !repeating 
    {
        ProgressStyle::with_template(
            "[{elapsed_precise}] {spinner:.blue} {bar:40.green/cyan} {pos:>0}/{len:>0} {msg}",
        )
        .unwrap()
        .tick_strings(&[
            "▹▹▹▹▹",
            "▸▹▹▹▹",
            "▹▸▹▹▹",
            "▹▹▸▹▹",
            "▹▹▹▸▹",
            "▹▹▹▹▸",
            "✅   ",
            ])
        .progress_chars("●●∙")
    }
    else
    {
        ProgressStyle::with_template(
            "[{elapsed_precise}] {spinner:.red}   {bar:40.green/cyan} {pos:>0}/{len:>0} {msg}",
        )
        .unwrap()
        .tick_strings(&[
            "∙∙∙",
			"●∙∙",
			"∙●∙",
			"∙∙●",
            "∙●∙",
			"●∙∙",
			"✅ "
            ])
        .progress_chars("●●∙")
    };
    pb.with_style(sty)
    
}


fn progress_bar_for_datetime(mpb: &MultiProgress, visible: bool, fp: &str, target_date: &Date, len: u32) -> ProgressBar
{
    let date = target_date.format(utilites::DateFormat::DotDate);
    let time = target_date.format(utilites::DateFormat::Time);
    let msg= if visible
    {
        [&date, " ", &time, " -> ", fp].concat()
    }
    else
    {
        [&date, " ", &time].concat()
    };
    let pb = mpb.add(ProgressBar::new(len as u64));
    pb.set_message(msg);
    pb.enable_steady_tick(Duration::from_millis(120));
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {spinner:.blue}    {bar:40.green/cyan} [{msg}]",
    )
    .unwrap()
    .tick_strings(&[
            "🕛",
			"🕐",
			"🕑",
			"🕒",
			"🕓",
			"🕔",
			"🕕",
			"🕖",
			"🕗",
			"🕘",
			"🕙",
			"🕚",
            "✅"
    ])
    .progress_chars("●●∙");
    pb.with_style(sty)
}

fn run_process(cfg: Config)
{
    let mpb = MultiProgress::default();
    let mut minutes: u32 = 0;
    let mut tasks = Vec::with_capacity(cfg.tasks.len());
    for t in cfg.tasks.iter()
    {
        if std::fs::exists(&t.file_path).is_ok_and(|f| f == true)
        {
            if let Some(i) = t.del_time_interval.as_ref()
            {
                let pb = progress_bar_for_interval(&mpb, t.repeat, t.visible, &t.file_path, *i);
                tasks.push((t, pb)); 
            }
            if let Some(d) = t.del_time.as_ref()
            {
                let now = Date::now();
                let target = time_diff(&now, d);
                let pb = progress_bar_for_datetime(&mpb, t.visible, &t.file_path, d, target as u32);
                tasks.push((t, pb)); 
            }
        }
        else 
        {
            println!("Ошибка, файл {} не существует, задача выполнена не будет", &t.file_path);
        }
    }
    while tasks.len() > 0
    {
        let mut del_tasks = Vec::new();
        for t in &tasks
        {
            if let Some(interval) = t.0.del_time_interval.as_ref()
            {
                if minutes != 0
                {
                    t.1.inc(1);
                    if  minutes % interval == 0
                    {
                        if t.0.repeat
                        {
                            t.1.reset();
                        }
                        else 
                        {
                            t.1.finish();
                        }
                        if del_file(&t.0.file_path) && !t.0.repeat
                        {
                            del_tasks.push(t.0.file_path.clone());
                        }
                    }
                }
            }
            else if let Some(checked_date) = t.0.del_time.as_ref()
            {
                let current_date = Date::now();
                let diff = time_diff(&current_date, checked_date);
                if diff.is_negative()
                {
                    if del_file(&t.0.file_path)
                    {
                        t.1.finish();
                        del_tasks.push(t.0.file_path.clone());
                    }
                }
                else 
                {
                    t.1.set_position(t.1.length().unwrap() - diff as u64);  
                }
            }
        }
        for d in del_tasks
        {
            tasks.retain(|r| r.0.file_path != d);
        }
        if tasks.len() > 0
        {
            std::thread::sleep(Duration::from_secs(60));
        }
        minutes += 1;
    }
}



///Получаем отрицательное значение если проверяемая дата меньше текущей
fn time_diff(current_date: &Date, checked_date: &Date) -> i64
{
    checked_date.as_naive_datetime().and_utc().timestamp() - current_date.as_naive_datetime().and_utc().timestamp()
}

fn del_file(path: &str) -> bool
{
    let del = std::fs::remove_file(path);
    if del.is_ok()
    {
        //println!("Файл {} удален", path);
        true
    }
    else 
    {
        //println!("Ошибка удаления файла {} ", path);
        false
    }
}


#[cfg(test)]
mod tests
{
    use utilites::Date;

    use crate::{structs::{Config, Task}, time_diff, FILE_NAME};

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
        let cfg = Config
        {
                tasks: vec![
                Task
                {
                    file_path: "/hard/xar/projects/tests/1".to_owned(),
                    del_time_interval: Some(2),
                    del_time: None,
                    repeat: false,
                    visible: true
                },
                Task
                {
                    file_path: "/hard/xar/projects/tests/2".to_owned(),
                    del_time_interval: None,
                    del_time: Some(Date::now().add_minutes(3)),
                    repeat: false,
                    visible: true
                },
                Task
                {
                    file_path: "/hard/xar/projects/tests/3".to_owned(),
                    del_time_interval: None,
                    del_time: Some(Date::now().add_minutes(6)),
                    repeat: false,
                    visible: true
                },
                Task
                {
                    file_path: "/hard/xar/projects/tests/4".to_owned(),
                    del_time_interval: Some(2),
                    del_time: None,
                    repeat: true,
                    visible: false
                },
            ]
        };
        let r = utilites::serialize(cfg, FILE_NAME, false, utilites::Serializer::Toml);
        //super::main();
        logger::info!("{:?}", r)
    }
}
