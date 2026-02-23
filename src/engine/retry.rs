use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

pub async fn with_retry<F, Fut, T, E>(name: &str, mut f: F, max_retries: usize) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) if attempt < max_retries => {
                attempt += 1;
                warn!(
                    operation = %name,
                    error = %e,
                    attempt = attempt,
                    next_retry_in = ?delay,
                    "Transient error, retrying"
                );
                sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
}
