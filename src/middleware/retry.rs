use std::time::Duration;

use reqwest::Client;
use tokio::time::sleep;

use crate::config::RetryConfig;

pub struct RetryClient {
    client: Client,
    config: RetryConfig,
}

impl RetryClient {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn request(
        &self,
        method: reqwest::Method,
        url: &str,
        headers: &reqwest::header::HeaderMap,
    ) -> Result<reqwest::Response, reqwest::Error> {
        if !self.config.enabled {
            return self.client
                .request(method, url)
                .headers(headers.clone())
                .send()
                .await;
        }

        let mut attempt = 0;
        let mut delay_ms = self.config.base_delay_ms;
        let jitter_max = 50;

        loop {
            attempt += 1;
            match self.client
                .request(method.clone(), url)
                .headers(headers.clone())
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if status < 500 || attempt >= self.config.max_attempts {
                        return Ok(resp);
                    }
                    tracing::warn!(attempt, status, "upstream returned {}, retrying", status);
                }
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        return Err(e);
                    }
                    tracing::warn!(attempt, error = %e, "upstream request failed, retrying");
                }
            }

            if attempt >= self.config.max_attempts {
                return self.client
                    .request(method, url)
                    .headers(headers.clone())
                    .send()
                    .await;
            }

            let jitter = rand::random::<u64>() % jitter_max;
            let delay = std::cmp::min(delay_ms + jitter, self.config.max_delay_ms);
            sleep(Duration::from_millis(delay)).await;
            delay_ms = delay_ms.saturating_mul(2);
        }
    }
}