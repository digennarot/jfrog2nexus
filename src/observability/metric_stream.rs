use bytes::Bytes;
use futures::Stream;
use metrics::counter;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A stream adapter that increments the `j2n_transfer_bytes_total` Prometheus counter
/// as bytes flow through it.
///
/// Wrap any `reqwest` byte stream with `MetricStream` to get per-repository bandwidth
/// metrics without modifying transfer logic.
pub struct MetricStream<S> {
    inner: S,
    repo_name: String,
}

impl<S> MetricStream<S> {
    /// Wrap `inner` in a `MetricStream` that labels metrics with `repo_name`.
    ///
    /// # Arguments
    ///
    /// * `inner` - The underlying byte stream to instrument.
    /// * `repo_name` - Repository label attached to the Prometheus counter.
    pub fn new(inner: S, repo_name: String) -> Self {
        Self { inner, repo_name }
    }
}

impl<S> Stream for MetricStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<Bytes, reqwest::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                let len = bytes.len() as u64;
                counter!("j2n_transfer_bytes_total", "repo" => self.repo_name.clone())
                    .increment(len);
                Poll::Ready(Some(Ok(bytes)))
            }
            res => res,
        }
    }
}
