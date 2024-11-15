// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use serde::Deserialize;

use crate::{CliError, SwayBarBlock};

// 成都三瓦窑
const URI: &str = "https://api.waqi.info/feed/@1362?token=";

#[derive(Debug, Clone)]
pub(crate) struct AqiFetcher {
    aqi: Arc<AtomicU32>,
}

impl AqiFetcher {
    pub(crate) async fn new() -> Result<Self, CliError> {
        let aqi = Arc::new(AtomicU32::new(0));
        let aqi_clone = aqi.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(i) = get_aqi().await {
                    aqi_clone.store(i, Ordering::Relaxed);
                }
                tokio::time::sleep(std::time::Duration::from_secs(1800)).await;
            }
        });
        Ok(Self { aqi })
    }

    pub(crate) fn get(&self) -> Option<SwayBarBlock> {
        let aqi: u32 = self.aqi.load(Ordering::Relaxed);

        if aqi > 0 {
            Some(SwayBarBlock {
                name: "aqi".into(),
                full_text: format!("AQI: {aqi}"),
                min_width: Some(9),
                ..Default::default()
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct AqiCnReply {
    data: AqiCnReplyData,
}

#[derive(Debug, Clone, Deserialize)]
struct AqiCnReplyData {
    aqi: u32,
}

async fn get_aqi() -> Result<u32, CliError> {
    let aqicn_key = if let Ok(k) = std::env::var("AQI_CN_KEY") {
        k
    } else {
        return Ok(0);
    };

    let body = reqwest::get(&format!("{URI}{aqicn_key}"))
        .await?
        .text()
        .await?;

    let reply: AqiCnReply = serde_json::from_str(&body)?;
    Ok(reply.data.aqi)
}

impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        Self {
            error_msg: format!("HTTP error: {e}"),
        }
    }
}
