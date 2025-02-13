use std::{collections::HashMap, fmt::format, hash::{Hash, Hasher}, path::Path, sync::Arc};

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

    pub async fn add_tasks(self, mpb: MultiProgress, tasks: Arc<RwLock<HashMap<u64, TaskWithProgress>>>, scheduler: Scheduler<u64>)
    {
        for task in self.tasks.into_iter()
        {
            let task = TaskWithProgress::new(task, &mpb);
            let task_id = task.get_hash();
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
                        let _ = scheduler.add_interval_task(task_id, i, repeating).await;
                    }
                    else if let Some(d) = task.get_date()
                    {
                        let _ = scheduler.add_date_task(task_id, d, repeating).await;
                    }
                }
                let mut guard = tasks.write().await;
                guard.insert(task_id, task);
            }
        }
    }
}
