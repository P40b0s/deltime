use std::{collections::HashMap, fmt::format, path::Path, sync::Arc};

use indicatif::MultiProgress;
use scheduler::Scheduler;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::structs::{Task, TaskWithProgress};

pub const FILE_NAME: &str = "config.toml";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config 
{
    pub tasks: Vec<Task>
}
impl Config
{
    pub fn load_local() -> Result<Self, crate::error::Error>
    {

        let config = utilites::deserialize::<Config, _>(FILE_NAME, false, utilites::Serializer::Toml)?;
        Ok(config)
    }
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, crate::error::Error>
    {

        let config = utilites::deserialize::<Config, _>(path, false, utilites::Serializer::Toml)?;
        Ok(config)
    }

    pub async fn add_tasks(self, mpb: MultiProgress, tasks: Arc<RwLock<HashMap<uuid::Uuid, TaskWithProgress>>>, scheduler: Arc<Scheduler<uuid::Uuid>>)
    {
        for task in self.tasks.into_iter()
        {
            let task = TaskWithProgress::new(task, &mpb);
            let task_id = uuid::Uuid::new_v4();
            let repeating = *task.get_strategy();
            if task.path_is_exists()
            {
                if let Some(i) = task.get_interval()
                {
                    let _ = scheduler.add_interval_task(task_id, i, repeating).await;
                }
                else if let Some(d) = task.get_date()
                {
                    let _ = scheduler.add_date_task(task_id, d, repeating).await;
                }
            }
            else 
            {
                //let _ = mpb.println(format!("файл не найден {}", task.get_str_path()));
            }
            let mut guard = tasks.write().await;
            guard.insert(task_id, task);
        }
    }
}
