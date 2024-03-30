// rate_limiter.rs
use std::fmt;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::Duration;

#[derive(Clone, Debug)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    period: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, period: Duration) -> Self {
        RateLimiter {
            semaphore: Arc::new(Semaphore::new(max_requests)),
            period,
        }
    }

    pub async fn acquire(&self) -> Result<(), RateLimitError> {
        let permit = self.semaphore.clone().acquire_owned().await;
        match permit {
            Ok(permit) => {
                let period = self.period;
                tokio::spawn(async move {
                    tokio::time::sleep(period).await;
                    drop(permit);
                });
                Ok(())
            }
            Err(_) => Err(RateLimitError::TooManyRequests),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitError {
    TooManyRequests,
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateLimitError::TooManyRequests => write!(f, "Too many requests"),
        }
    }
}

impl std::error::Error for RateLimitError {}
