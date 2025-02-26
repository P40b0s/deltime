use std::{future::Future, pin::Pin, task::{Context, Poll}};
use actually_beep::beep_with_hz_and_millis;

enum PlayState
{
    PlayingOk,
    #[allow(dead_code)]
    PlayingError,
    Done
}

pub struct Beeper
{
    state: PlayState
}
impl Beeper
{
    fn play_sound(hz: u32)
    {
        let duration = 200;
        let _ = beep_with_hz_and_millis(hz, duration);
    }

    pub fn ok() -> Self
    {
        Self
        {
            state: PlayState::PlayingOk
        }
    }
    #[allow(dead_code)]
    pub fn error() -> Self
    {
        Self
        {
            state: PlayState::PlayingError
        }
    }
}
impl Future for Beeper
{
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        match &mut self.state
        {
            PlayState::PlayingOk =>
            {
                let walker = cx.waker().clone();
                self.state = PlayState::Done;
                tokio::task::spawn_blocking(||
                {
                    Beeper::play_sound(1900);
                    Beeper::play_sound(2600);
                    walker.wake();
                });
               Poll::Pending
            },
            PlayState::PlayingError => 
            {
                let walker = cx.waker().clone();
                self.state = PlayState::Done;
                tokio::task::spawn_blocking(||
                {
                    Beeper::play_sound(600);
                    Beeper::play_sound(400);
                    Beeper::play_sound(100);
                    walker.wake();
                });
                Poll::Pending
            },
            PlayState::Done => Poll::Ready(())
        }
        
    }
}


#[cfg(test)]
mod tests
{
    use crate::beeper::Beeper;
    #[tokio::test]
    async fn test_async()
    {
        let beeper_ok = Beeper::ok();
        beeper_ok.await;
        Beeper::error().await;
    }
}