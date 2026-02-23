use bytes::Bytes;
use futures::Stream;
use governor::{clock::DefaultClock, state::InMemoryState, state::NotKeyed, Quota, RateLimiter};
use nonzero_ext::nonzero;
use std::future::Future;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub type GlobalRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

pub struct ThrottledStream<S> {
    inner: S,
    limiter: Arc<GlobalRateLimiter>,
    delay: Option<Pin<Box<tokio::time::Sleep>>>,
    pending_bytes: Option<Bytes>,
}

impl<S> ThrottledStream<S> {
    pub fn new(inner: S, limiter: Arc<GlobalRateLimiter>) -> Self {
        Self {
            inner,
            limiter,
            delay: None,
            pending_bytes: None,
        }
    }
}

impl<S> Stream for ThrottledStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<Bytes, reqwest::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // 1. If we have a delay active, poll it
        if let Some(mut delay) = self.delay.take() {
            match delay.as_mut().poll(cx) {
                Poll::Ready(_) => {
                    // Delay finished, fall through to check pending bytes
                }
                Poll::Pending => {
                    self.delay = Some(delay);
                    return Poll::Pending;
                }
            }
        }

        // 2. If we have pending bytes (rate limited last time), try to push them now
        let bytes = if let Some(b) = self.pending_bytes.take() {
            b
        } else {
            // 3. Otherwise, get next chunk from inner
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(b))) => b,
                res => return res,
            }
        };

        let len = bytes.len() as u32;
        if len == 0 {
            return Poll::Ready(Some(Ok(bytes)));
        }

        match self
            .limiter
            .check_n(NonZeroU32::new(len).unwrap_or(nonzero!(1u32)))
        {
            Ok(_) => Poll::Ready(Some(Ok(bytes))),
            Err(_) => {
                // Rate limited. Store bytes and set a delay.
                self.pending_bytes = Some(bytes);
                self.delay = Some(Box::pin(tokio::time::sleep(
                    std::time::Duration::from_millis(100),
                )));

                // Poll the newly created delay immediately to register waker
                if let Some(mut delay) = self.delay.take() {
                    let _ = delay.as_mut().poll(cx);
                    self.delay = Some(delay);
                }

                Poll::Pending
            }
        }
    }
}

pub fn create_limiter(kbps: u64) -> Arc<GlobalRateLimiter> {
    let quota = Quota::per_second(NonZeroU32::new((kbps * 1024) as u32).unwrap_or(nonzero!(1u32)));
    Arc::new(RateLimiter::direct(quota))
}
