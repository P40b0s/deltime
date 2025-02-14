use std::{collections::HashMap, sync::Arc};
use scheduler::{SchedulerEvent, SchedulerHandler};
use tokio::sync::RwLock;
use crate::structs::TaskWithProgress;


pub struct Handler
{
    tasks: Arc<RwLock<HashMap<Arc<String>, TaskWithProgress>>>
}
impl Handler
{
    pub fn new(tasks: Arc<RwLock<HashMap<Arc<String>, TaskWithProgress>>>) -> Self
    {
        Self
        {
            tasks
        }
    }
}
impl SchedulerHandler<Arc<String>> for Handler
{
    fn tick(&self, event: scheduler::SchedulerEvent<Arc<String>>) -> impl std::future::Future<Output = ()> 
    {
        let task = self.tasks.clone();
        async move
        {
            match event
            {
                SchedulerEvent::Tick(event) => 
                {
                    logger::debug!("tick event_id: {:?}", &event);
                    let guard = task.read().await;
                    if let Some(t) = guard.get(&event.id)
                    {
                        t.update_progress(event.current as u64, event.len as u64);
                    }
                },
                SchedulerEvent::Expired(event) => 
                {
                    logger::debug!("expired event_id: {:?}", &event);
                    let guard = task.read().await;
                    if let Some(t) = guard.get(&event)
                    {
                        t.finish_with_err(["Время операции c `", t.get_str_path(), "` уже прошло"].concat());
                    }
                },
                SchedulerEvent::Finish(event) =>
                {
                    let guard = task.read().await;
                    logger::debug!("finish event_id: {:?}", &event);
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
                    logger::debug!("finish_cycle event_id: {:?}", &event);
                    let mut guard = task.write().await;
                    if let Some(t) = guard.get_mut(&event.id)
                    {
                        t.update_progress_with_cycle(event.current as u64, event.len as u64);
                        let _ = t.del_file().await;
                    }
                }
            };
        }
    }
}