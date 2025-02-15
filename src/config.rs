use std::{collections::HashMap, fmt::format, hash::{Hash, Hasher}, path::Path, sync::Arc};

use indicatif::MultiProgress;
use scheduler::{RepeatingStrategy, Scheduler};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{beeper, structs::{Task, TaskWithProgress}};

pub const FILE_NAME: &str = "config.toml";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config 
{
    pub tasks: Vec<Task>
}
impl Config
{
    pub async fn load() -> Self
    {
        if let Ok(config) = Config::load_local()
        {
            config
        }
        else 
        {
            logger::warn!("Локальный файл конфигурации {} не обнаружен, ожидаю ввода...", FILE_NAME);
            #[cfg(feature="beeper")]
            beeper::Beeper::ok().await;
            Config
            {
                tasks: Vec::new()
            }
        }
    }
    fn load_local() -> Result<Self, crate::error::Error>
    {

        let config = utilites::deserialize::<Config, _>(FILE_NAME, false, utilites::Serializer::Toml)?;
        Ok(config)
    }
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, crate::error::Error>
    {

        let config = utilites::deserialize::<Config, _>(path, false, utilites::Serializer::Toml)?;
        Ok(config)
    }

    pub async fn add_tasks(self, mpb: MultiProgress, tasks: Arc<RwLock<HashMap<Arc<String>, TaskWithProgress>>>, scheduler: Scheduler<Arc<String>>)
    {
        #[cfg(feature="beeper")]
        super::beeper::Beeper::ok().await;
        for task in self.tasks.into_iter()
        {
            let task_id = Arc::new(task.get_hash());
            let task = TaskWithProgress::new(task, &mpb);
            logger::debug!("new task fom config: {:?} id: {}", &task, &task_id);
            let exists = 
            {
                let guard = tasks.read().await;
                guard.contains_key(&task_id)
            };
            if !exists
            {
                let repeating = *task.get_strategy();
                if task.path_is_exists()
                {
                    if let Some(i) = task.get_interval()
                    {
                        if scheduler.add_interval_task(task_id.clone(), i, repeating).await
                        {
                            let mut guard = tasks.write().await;
                            guard.insert(task_id, task);
                        }
                        else 
                        {
                            task.finish_with_err(["Ошибка добавления задачи ", task.get_str_path()].concat());    
                        }
                    }
                    else if let Some(d) = task.get_date()
                    {
                        if scheduler.add_date_task(task_id.clone(), d, repeating).await
                        {
                            let mut guard = tasks.write().await;
                            guard.insert(task_id, task);
                        }
                        else 
                        {
                            task.finish_with_err(["Ошибка добавления задачи ", task.get_str_path()].concat());    
                        }
                    }
                }
            }
        }
    }
}
