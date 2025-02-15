use std::{future::Future, pin::Pin, task::{Context, Poll}};

use actually_beep::beep_with_hz_and_millis;

fn play_sound(hz: u32)
{
    let middle_e_hz = hz;
    let a_bit_more_than_a_second_and_a_half_ms = 200;
    beep_with_hz_and_millis(middle_e_hz, a_bit_more_than_a_second_and_a_half_ms).unwrap();
}

fn ok_sound()
{
    play_sound(1900);
    play_sound(2600);
}
fn error_sound()
{
    play_sound(600);
    play_sound(400);
    play_sound(100);
}

enum PlayState
{
    PlayingOk,
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

    #[test]
    fn test_beep()
    {
        super::play_sound(1900);
        super::play_sound(2600);
    }
    #[test]
    fn test_beep2()
    {
        super::play_sound(600);
        super::play_sound(400);
        super::play_sound(100);
    }
    #[tokio::test]
    async fn test_async()
    {
        let beeper_ok = Beeper::ok();
        beeper_ok.await;
        Beeper::error().await;
    }
}