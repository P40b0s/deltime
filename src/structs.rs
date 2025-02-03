use super::cli::Cli;
use serde::{Deserialize, Serialize};
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
    pub file_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub del_time_interval: Option<u32>,
    #[serde(deserialize_with="deserialize_data")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub del_time: Option<Date>,
    #[serde(default)]
    pub repeat: bool,
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

impl TryInto<Config> for Cli
{
    type Error = String;
    fn try_into(self) -> Result<Config, Self::Error> 
    {
        let mut date: Option<Date> = None;
        if self.date.is_none() && self.interval.is_none()
        {
            return Err("Не переданы аргументы времени, необходимо передать -i или -d".to_string());
        }
        if self.date.is_some() && self.interval.is_some()
        {
            return Err("Можно передеть только один временной параметр или -i или -d".to_string());
        }
        let interval = self.interval;
        let repeat = if interval.is_some() && self.repeat
        {
            self.repeat
        }
        else
        {
            false
        };
        if let Some(d) = self.date
        {
            if let Some(d) = Date::parse(d)
            {
                date = Some(d);
            }   
            else 
            {
                return Err("Ошибка формата даты в аргументе -d".to_string());
            }
        }
        if std::fs::exists(&self.file).is_ok_and(|f| f == true)
        {
            return  Ok(Config
            {
                tasks: vec![
                    Task
                    {
                        file_path: self.file,
                        del_time: date,
                        del_time_interval: interval,
                        repeat,
                        visible: self.visible
                    }
                ]
            })
        }
        else 
        {
            return Err("Ошибка, указанный файл не существует".to_string());
        };
        
    }
}