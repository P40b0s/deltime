use std::{pin::Pin, task::{Context, Poll}};

use futures::Stream;
use tokio::sync::mpsc::Receiver;
use utilites::Date;

pub fn time_diff(current_date: &Date, checked_date: &Date) -> i64
{
    checked_date.as_naive_datetime().and_utc().timestamp() - current_date.as_naive_datetime().and_utc().timestamp()
}

#[derive(Debug)]
pub struct ReceiverStream<T> 
{
    inner: Receiver<T>,
}

impl<T> ReceiverStream<T> 
{
    /// Create a new `ReceiverStream`.
    pub fn new(recv: Receiver<T>) -> Self 
    {
        Self { inner: recv }
    }

    /// Get back the inner `Receiver`.
    pub fn into_inner(self) -> Receiver<T> 
    {
        self.inner
    }

    /// Closes the receiving half of a channel without dropping it.
    ///
    /// This prevents any further messages from being sent on the channel while
    /// still enabling the receiver to drain messages that are buffered. Any
    /// outstanding [`Permit`] values will still be able to send messages.
    ///
    /// To guarantee no messages are dropped, after calling `close()`, you must
    /// receive all items from the stream until `None` is returned.
    ///
    /// [`Permit`]: struct@tokio::sync::mpsc::Permit
    pub fn close(&mut self) 
    {
        self.inner.close();
    }
}

impl<T> Stream for ReceiverStream<T> 
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> 
    {
        self.inner.poll_recv(cx)
    }
}

impl<T> AsRef<Receiver<T>> for ReceiverStream<T> 
{
    fn as_ref(&self) -> &Receiver<T> 
    {
        &self.inner
    }
}

impl<T> AsMut<Receiver<T>> for ReceiverStream<T> 
{
    fn as_mut(&mut self) -> &mut Receiver<T> 
    {
        &mut self.inner
    }
}

impl<T> From<Receiver<T>> for ReceiverStream<T> 
{
    fn from(recv: Receiver<T>) -> Self 
    {
        Self::new(recv)
    }
}


#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct UnalignedU64 
{
    value: [u8; 8],
}

impl UnalignedU64 
{
    pub fn new(value: u64) -> Self 
    {
        Self 
        {
            value: u64::to_ne_bytes(value)
        }
    }
    pub fn to_u64(self) -> u64 
    {
        u64::from_ne_bytes(self.value)
    }
    
    pub fn from_u8_array(array: &[u8]) -> &[UnalignedU64] 
    {
        let len = array.len() / 8;
        let ptr = array.as_ptr() as *const UnalignedU64;
        unsafe 
        {
            std::slice::from_raw_parts(ptr, len)
        }
    }
}