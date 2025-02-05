use std::borrow::Cow;
use crate::tasker::RepeatingStrategy;
use crate::helpers::time_diff;
use indicatif::{MultiProgress, ProgressBar};
use serde::{Deserialize, Serialize, Serializer};
use utilites::Date;

pub trait DeleteTaskTrait
{
    fn get_path(&self) -> &str;
    fn get_date(&self) -> &Option<Date>;
    fn get_interval(&self) -> Option<u32>;
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Task
{
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    #[serde(deserialize_with="deserialize_data")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<Date>,
    #[serde(serialize_with="serialize_repeating")]
    #[serde(deserialize_with="deserialize_repeating")]
    pub repeat: RepeatingStrategy,
    #[serde(default)]
    pub visible: bool
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config 
{
    pub tasks: Vec<Task>
}

fn deserialize_data<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer)?;
    if let Some(date) = Date::parse(&s)
    {
        Ok(Some(date))
    }
    else 
    {
        Err(serde::de::Error::custom(["Ошибка формата даты ", &s].concat()))
    }
}

fn deserialize_repeating<'de, D>(deserializer: D) -> Result<RepeatingStrategy, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer)?;
    match s.as_str()
    {
        "monthly" => Ok(RepeatingStrategy::Monthly),
        "dialy" => Ok(RepeatingStrategy::Dialy),
        "forever" => Ok(RepeatingStrategy::Forever),
        "once" => Ok(RepeatingStrategy::Once),
        _ => Err(serde::de::Error::custom(["Ошибка, опции `" , &s, "` не существует"].concat()))
    }
}
fn serialize_repeating<S>(repeat: &RepeatingStrategy, serializer: S) -> Result<S::Ok, S::Error> 
where 
    S: Serializer,
{
    serializer.serialize_str(&repeat.to_string())
}

#[derive(Clone, Debug)]
pub struct TaskWithProgress
{
    task: Task,
    pb: ProgressBar
}
impl TaskWithProgress
{
    pub fn new(task: Task, mpb: &MultiProgress) -> Self
    {
        let pb = if std::fs::exists(&task.path).is_ok_and(|f| f == true)
        {
            if let Some(d) = task.date.as_ref()
            {
                let now = Date::now();
                let target = time_diff(&now, &d);
                let pb = crate::progress_bar_for_datetime(mpb, target as u32);
                Self::set_date_message(&pb, task.visible, d, &task.path,task.mask.as_ref(), &task.repeat);
                pb
            }
            else if let Some(i) = task.interval
            {
                let pb = crate::progress_bar_for_interval(mpb, &task.repeat, i);
                Self::set_interval_message(&pb, task.visible, &task.path,task.mask.as_ref(), &task.repeat);
                pb
            }
            else
            {
                ProgressBar::hidden()
            }
        }
        else 
        {
            let pb = crate::progress_bar_for_interval(mpb, &RepeatingStrategy::Once,0);
            pb.set_prefix("❌");
            pb.set_message(["файл ", &task.path, " не существует"].concat());
            pb.finish();
            pb
        };
        Self
        {
            task,
            pb
        }
    }
    pub fn get_interval(&self) -> Option<u32>
    {
        self.task.interval
    }
    pub fn get_date(&self) -> Option<Date>
    {
        self.task.date.clone()
    }
    // pub fn get_time_diff(&self) -> Option<i64>
    // {
    //     if let Some(date) = self.task.date.as_ref()
    //     {
    //         let now = Date::now();
    //         Some(time_diff(&now, date))
    //     }
    //     else 
    //     {
    //         None
    //     }
    // }
    pub fn path_is_exists(&self) -> bool
    {
        std::fs::exists(&self.task.path).is_ok_and(|f| f == true)
    }
    pub fn get_path(&self) -> &str
    {
        &self.task.path
    }
    pub fn get_strategy(&self) -> &RepeatingStrategy
    {
        &self.task.repeat
    }
    pub fn set_prefix(&self, prefix: impl Into<Cow<'static, str>>)
    {
        self.pb.set_prefix(prefix);
    }
    // pub fn set_message(&self, message: impl Into<Cow<'static, str>>)
    // {
    //     self.pb.set_message(message);
    // }
    pub fn print_line<P: AsRef<str>>(&self, message: P)
    {
        self.pb.println(message);
    }
    ///finish progressbar work
    pub fn finish(&self)
    {
        self.set_prefix("✅");
        self.pb.finish();
    }
    pub fn finish_with_err<P: AsRef<str>>(&self, err: P)
    {
        self.set_prefix("❌");
        self.print_line(err);
        self.pb.finish();
    }
    ///reset progressbar
    pub fn reset(&self)
    {
        self.pb.reset();
    }
    pub fn update_progress(&self)
    {
        if let Some(date) = self.task.date.as_ref()
        {
            let current_date = Date::now();
            let diff = time_diff(&current_date, date);
            if diff.is_positive()
            {
                self.pb.set_position(self.pb.length().unwrap() - diff as u64);  
            }
        }
        else if let Some(_) = self.task.interval.as_ref()
        {
            self.pb.inc(1);
        }
    }
    pub fn update_progress_with_cycle(&mut self, new_date: Option<Date>)
    {
        self.reset();
        if self.task.date.is_some()
        {
            if let Some(new_date) = new_date
            {
                let current_date = Date::now();
                let new_len = time_diff(&current_date, &new_date);
                self.pb.set_length(new_len as u64);
                Self::set_date_message(&self.pb, self.task.visible, &new_date, self.get_path(), self.task.mask.as_ref(), self.get_strategy());
                self.task.date = Some(new_date);
            }
        }
    }

    pub async fn del_file(&self) -> Result<(), String>
    {
        let path = self.get_path();
        if !self.path_is_exists()
        {
            return Err(["Файл `", path, "` не найден"].concat());
        }
        let metadata = tokio::fs::metadata(path).await;
        if let Ok(md) = metadata
        {
            if md.is_file()
            {
                let del = tokio::fs::remove_file(path).await;
                return if del.is_ok()
                {
                    Ok(()) 
                }
                else 
                {
                    match del.err().unwrap().kind()
                    {
                        tokio::io::ErrorKind::PermissionDenied | tokio::io::ErrorKind::ResourceBusy =>
                            Err(["Нет прав или файл `", path, "` занят другим приложением"].concat()),
                        tokio::io::ErrorKind::NotFound =>
                            Err(["Файл `", path, "` не найден"].concat()),
                        _=> Ok(())
                    }
                };
            }
            if md.is_dir()
            {
                if let Some(mask ) = self.task.mask.as_ref()
                {
                    return if let Ok(files) = utilites::io::get_files_by_mask(path, mask).await
                    {
                        for f in files
                        {
                            let _ = tokio::fs::remove_file(&f).await;
                        }
                        Ok(())
                    }
                    else 
                    {
                        Err(["При операции c `", path, "` произошла ошибка"].concat())
                    }
                }
                else 
                {
                    let del = tokio::fs::remove_dir_all(path).await;
                    return if del.is_ok()
                    {
                        Ok(())   
                    }
                    else 
                    {
                        match del.err().unwrap().kind()
                        {
                            tokio::io::ErrorKind::PermissionDenied | tokio::io::ErrorKind::ResourceBusy =>
                                Err(["Нет прав или файл `", path, "` занят другим приложением"].concat()),
                            tokio::io::ErrorKind::NotFound =>
                                Err(["Файл `", path, "` не найден"].concat()),
                            _=> Ok(())
                        }
                    };
                }
            }
        }
        Err("Ошибка получения метадаты".to_owned())
        
    }
    fn set_date_message(pb: &ProgressBar, visible: bool, date: &Date, path: &str, mask: Option<&String>, strategy: &RepeatingStrategy)
    {
        let d = date.format(utilites::DateFormat::DotDate);
        let t = date.format(utilites::DateFormat::Time);
        let msg= if visible
        {
            if let Some(m) = mask
            {
                [&d, " ", &t, " -> ", path, " (", m, ")"].concat()
            }
            else
            {
                [&d, " ", &t, " -> ", path].concat()
            }
        }
        else
        {
            [&d, " ", &t].concat()
        };
        pb.set_message(msg);
        if Self::is_run_forever(strategy)
        {
            pb.set_prefix("🔃");
        }
        else 
        {
            pb.set_prefix("⌛");
        }
    }

    fn set_interval_message(pb: &ProgressBar, visible: bool, path: &str, mask: Option<&String>, strategy: &RepeatingStrategy)
    {
        let msg= if visible
        {
            if let Some(m) = mask
            {
                [" -> ", path, " (", m, ")"].concat()
            }
            else
            {
                [" -> ", path].concat()
            }
        }
        else
        {
            "".to_owned()
        };
        pb.set_message(msg);
        if Self::is_run_forever(strategy)
        {
            pb.set_prefix("🔃");
        }
        else 
        {
            pb.set_prefix("⌛");
        }
    }
    pub fn is_run_forever(strategy: &RepeatingStrategy) -> bool
    {
        if let RepeatingStrategy::Forever | RepeatingStrategy::Dialy | RepeatingStrategy::Monthly = *strategy
        {
            true
        }
        else 
        {
            false    
        }
    }
}