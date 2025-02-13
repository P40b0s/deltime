use std::{collections::HashMap, sync::Arc};
use scheduler::{SchedulerEvent, SchedulerHandler};
use tokio::sync::RwLock;
use crate::structs::TaskWithProgress;


pub struct Handler
{
    tasks: Arc<RwLock<HashMap<u64, TaskWithProgress>>>
}
impl Handler
{
    pub fn new(tasks: Arc<RwLock<HashMap<u64, TaskWithProgress>>>) -> Self
    {
        Self
        {
            tasks
        }
    }
}
impl SchedulerHandler<u64> for Handler
{
    fn tick(&self, event: scheduler::SchedulerEvent<u64>) -> impl std::future::Future<Output = ()> 
    {
        let task = self.tasks.clone();
        async move
        {
            match event
            {
                SchedulerEvent::Tick(event) => 
                {
                    let guard = task.read().await;
                    if let Some(t) = guard.get(&event.id)
                    {
                        t.update_progress(event.current as u64, event.len as u64);
                    }
                },
                SchedulerEvent::Expired(event) => 
                {
                    let guard = task.read().await;
                    if let Some(t) = guard.get(&event)
                    {
                        t.finish_with_err(["Время операции c `", t.get_str_path(), "` уже прошло"].concat());
                    }
                },
                SchedulerEvent::Finish(event) =>
                {
                    let guard = task.read().await;
                    if let Some(t) = guard.get(&event)
                    {
                        if let Err(e) = t.del_file().await
                        {
                            t.finish_with_err(e);
                        }
                        else 
                        {
                            t.finish();
                        }
                    }
                },
                SchedulerEvent::FinishCycle(event) =>
                {
                    let mut guard = task.write().await;
                    if let Some(t) = guard.get_mut(&event.id)
                    {
                        t.update_progress_with_cycle(event.current as u64, event.len as u64);
                        let r = t.del_file().await;
                        logger::error!("{:?}", r);
                    }
                }
            };
        }
    }
}