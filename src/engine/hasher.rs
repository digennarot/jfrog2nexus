use bytes::Bytes;
use futures::Stream;
use sha2::{Digest, Sha256};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

pub struct HashingStream<S> {
    inner: S,
    hasher: Arc<Mutex<Sha256>>,
}

impl<S> HashingStream<S> {
    pub fn new(inner: S) -> (Self, Arc<Mutex<Sha256>>) {
        let hasher = Arc::new(Mutex::new(Sha256::new()));
        (
            Self {
                inner,
                hasher: hasher.clone(),
            },
            hasher,
        )
    }
}

impl<S> Stream for HashingStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<Bytes, reqwest::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                let mut hasher = self.hasher.lock().unwrap();
                hasher.update(&bytes);
                Poll::Ready(Some(Ok(bytes)))
            }
            res => res,
        }
    }
}
