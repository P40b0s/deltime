use std::{collections::HashMap, fmt::{Debug, Display}, ops::{Deref, DerefMut}, sync::{Arc, LazyLock}};
use scheduler::{SchedulerEvent, SchedulerHandler};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::{Receiver, Sender, UnboundedReceiver, UnboundedSender}, RwLock};
use utilites::Date;
use crate::{helpers::time_diff, structs::TaskWithProgress};


pub struct Handler
{
    tasks: Arc<RwLock<HashMap<uuid::Uuid, TaskWithProgress>>>
}
impl Handler
{
    pub fn new(tasks: HashMap<uuid::Uuid, TaskWithProgress>) -> Self
    {
        Self
        {
            tasks: Arc::new(RwLock::new(tasks))
        }
    }
}
impl SchedulerHandler<uuid::Uuid> for Handler
{
    fn tick(&self, event: scheduler::SchedulerEvent<uuid::Uuid>) -> impl std::future::Future<Output = ()> 
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
                        let _ = t.del_file().await;
                    }
                }
            };
        }
    }
}