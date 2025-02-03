use std::{fmt::Debug, ops::{Deref, DerefMut}, sync::{Arc, LazyLock}};
use tokio::sync::{mpsc::{Receiver, Sender, UnboundedReceiver, UnboundedSender}, RwLock};
use utilites::Date;

//static TASKER: LazyLock<UnboundedReceiver> = LazyLock::new(|| Arc::new(RwLock::new(Tasker::new())));
#[derive(Debug)]
pub struct TimerTask<T: Debug> where T: Send + Sync + Clone
{
    ///интервал запуска в минутах
    pub interval: Option<u32>,
    ///точное время удаления
    pub time: Option<Date>,
    ///Стратегия повтора задачи
    pub repeating_strategy: RepeatingStrategy,
    pub finished: bool,
    pub object: T
}
#[derive(Debug)]
pub struct Tasker<T>(Arc<RwLock<Vec<TimerTask<T>>>>) where T: Send + Sync + Clone + Debug;
#[derive(Clone, Debug)]
pub enum RepeatingStrategy
{
    Once,
    Forever,
    Dialy,
    Monthly
}

#[derive(Clone, Debug)]
pub enum ProcessStatus<T> where T: Send + Sync + Clone + Debug
{
    Tick(T),
    Finish(T),
    FinishCycle((T, Option<Date>))
}


impl<T> Tasker<T> where T: Send + Sync + Clone + Debug
{
    pub fn new() -> Self
    {
        Self(Arc::new(RwLock::new(Vec::new())))
    }
    pub fn get_channel(&self) -> (Sender<ProcessStatus<T>>, Receiver<ProcessStatus<T>>)
    {
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ProcessStatus<T>>(10);
        (sender, receiver)
    }

    pub async fn run(&self, sender: Sender<ProcessStatus<T>>)
    {
        let mut minutes: u32 = 0;
        loop 
        {
            let mut guard = self.0.write().await;
            for t in guard.iter_mut()
            {
                if !t.finished
                {
                    match t.repeating_strategy
                    {
                        RepeatingStrategy::Once =>
                        {
                            if let Some(interval) = t.interval.as_ref()
                            {
                                if minutes != 0 && (minutes % interval == 0)
                                {
                                    let _ = sender.send(ProcessStatus::Finish(t.object.clone())).await;
                                    t.finished = true;
                                }
                                else 
                                {
                                    let _ = sender.send(ProcessStatus::Tick(t.object.clone())).await;
                                }
                            }
                            else if let Some(time) = t.time.as_ref()
                            {
                                let current_date = Date::now();
                                let diff = time_diff(&current_date, time);
                                if diff.is_negative()
                                {
                                    let _ = sender.send(ProcessStatus::Finish(t.object.clone())).await;
                                    t.finished = true;
                                }
                                else 
                                {
                                    let _ = sender.send(ProcessStatus::Tick(t.object.clone())).await;
                                }
                            }
                        },
                        RepeatingStrategy::Forever =>
                        {
                            if let Some(interval) = t.interval.as_ref()
                            {
                                if minutes != 0 && (minutes % interval == 0)
                                {
                                    let _ = sender.send(ProcessStatus::FinishCycle((t.object.clone(), None))).await;
                                }
                                else 
                                {
                                    let _ = sender.send(ProcessStatus::Tick(t.object.clone())).await;
                                }
                            }
                            else if let Some(time) = t.time.as_ref()
                            {
                                let current_date = Date::now();
                                let diff = time_diff(&current_date, time);
                                if diff.is_negative()
                                {
                                    //делаем текущую дату добавляем время из старой даты и добаляем 1 сутки
                                    let new_date = Date::now().with_time(time).add_minutes(24*60);
                                    t.time = Some(new_date);
                                    let _ = sender.send(ProcessStatus::FinishCycle((t.object.clone(), t.time.clone()))).await;
                                }
                                else 
                                {
                                    let _ = sender.send(ProcessStatus::Tick(t.object.clone())).await;
                                }
                            }
                        },
                        RepeatingStrategy::Dialy =>
                        {
                            if let Some(time) = t.time.as_ref()
                            {
                                let current_date = Date::now();
                                let diff = time_diff(&current_date, time);
                                if diff.is_negative()
                                {
                                    //делаем текущую дату добавляем время из старой даты и добаляем 1 сутки
                                    let new_date = Date::now().with_time(time).add_minutes(24*60);
                                    t.time = Some(new_date);
                                    let _ = sender.send(ProcessStatus::FinishCycle((t.object.clone(), t.time.clone()))).await;
                                }
                                else 
                                {
                                    let _ = sender.send(ProcessStatus::Tick(t.object.clone())).await;
                                }
                            }
                        },
                        RepeatingStrategy::Monthly =>
                        {
                            if let Some(time) = t.time.as_ref()
                            {
                                let current_date = Date::now();
                                let diff = time_diff(&current_date, time);
                                if diff.is_negative()
                                {
                                    if let Some(new_date) = Date::now().with_time(time).add_months(1)
                                    {
                                        t.time = Some(new_date);
                                    }
                                    else 
                                    {
                                        t.finished = true;    
                                    }
                                    let _ = sender.send(ProcessStatus::FinishCycle((t.object.clone(), t.time.clone()))).await;
                                }
                                else 
                                {
                                    let _ = sender.send(ProcessStatus::Tick(t.object.clone())).await;
                                }
                            }
                        }
                    }
                }
            }
           
            guard.retain(|d| !d.finished);
            logger::debug!("{}", &guard.len());
            if guard.len() == 3
            {
                for t in guard.iter()
                {
                    logger::debug!("{:?}", t);
                }
            }
            drop(guard);
            tokio::time::sleep(tokio::time::Duration::from_millis(60000)).await;
            //tick on early minute
            //let _ = sender.send(None).await;             
            minutes += 1;
        }
    }

    pub async fn add_interval_task(&self, task: T, interval: u32, repeat: bool)
    {
        let strategy  = if repeat
        {
            RepeatingStrategy::Forever
        }
        else 
        {
            RepeatingStrategy::Once    
        };
        let task = TimerTask
        {
            interval: Some(interval),
            time: None,
            repeating_strategy: strategy,
            finished: false,
            object: task
        };
        let mut guard = self.0.write().await;
        guard.push(task);
    }

    pub async fn add_date_task(&self, task: T, date: &Date, repeating_strategy: RepeatingStrategy)
    {
        let mut guard = self.0.write().await;
        match &repeating_strategy 
        {
            _ =>
            {
                
                let task = TimerTask
                {
                    interval: None,
                    time: Some(date.clone()),
                    repeating_strategy,
                    finished: false,
                    object: task
                };
                guard.push(task);
            }
        };
    }
}


fn current_timestramp(date: &Date) -> u32
{
    let now = Date::now();
    let target = time_diff(&now, date);
    target as u32
}

fn time_diff(current_date: &Date, checked_date: &Date) -> i64
{
    checked_date.as_naive_datetime().and_utc().timestamp() - current_date.as_naive_datetime().and_utc().timestamp()
}
