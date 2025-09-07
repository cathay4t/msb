// SPDX-License-Identifier: Apache-2.0

mod aqi;
mod battery;
mod cpu;
mod error;
mod fs;
mod rate;
mod sound;
mod temp;
mod wifi;

use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::error::CliError;

pub(crate) const COLOR_RED: &str = "#FF0000";
pub(crate) const COLOR_YELLOW: &str = "#E9F505";
// pub(crate) const COLOR_GREEN: &str = "#00FF00";

const IFACE_NAME: &str = "wlan0";
const INTERVAL: u64 = 10;

// Following manpage swaybar-protocol

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct SwayBarApiVersion {
    version: u8,
    click_events: bool,
}

impl Default for SwayBarApiVersion {
    fn default() -> Self {
        SwayBarApiVersion {
            version: 1,
            click_events: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
struct SwayBarBlock {
    name: String,
    full_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_width: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    urgent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

const SIGCONT: std::ffi::c_int = 18;

fn log(line: &str) -> Result<(), CliError> {
    let mut fd = std::fs::File::options()
        .create(true)
        .append(true)
        .open("/tmp/msb.log")?;
    Ok(writeln!(fd, "{line}")?)
}

fn get_time() -> SwayBarBlock {
    let now = chrono::offset::Local::now();
    SwayBarBlock {
        name: "time".to_string(),
        full_text: format!("{}", now.format("%Y-%m-%d %H:%M:%S")),
        ..Default::default()
    }
}

async fn emit_status(
    aqi_fether: &crate::aqi::AqiFetcher,
) -> Result<(), CliError> {
    let mut blocks: Vec<SwayBarBlock> = Vec::new();

    if let Some(b) = aqi_fether.get() {
        blocks.push(b);
    }
    blocks.push(crate::rate::get_rate(IFACE_NAME).await?);
    blocks.push(crate::wifi::get_wifi(IFACE_NAME).await?);
    blocks.push(crate::cpu::get_cpu().await?);
    for block in crate::temp::get_temp()? {
        blocks.push(block);
    }
    blocks.push(crate::sound::get_sound()?);
    if let Some(b) = crate::battery::get_battery()? {
        blocks.push(b);
    }
    blocks.push(get_time());

    println!("{}", serde_json::to_string_pretty(&blocks)?);
    print!(",");
    Ok(())
}

#[tokio::main()]
async fn main() -> Result<(), CliError> {
    let mut continue_stream = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::from_raw(SIGCONT),
    )
    .map_err(|e| format!("tokio failed to hook on signal SIGCONT: {e}"))?;

    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(INTERVAL));

    println!("{}", serde_json::to_string(&SwayBarApiVersion::default())?);

    println!("[");

    let aqi_fether = crate::aqi::AqiFetcher::new().await?;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = emit_status(&aqi_fether).await {
                    log(&e.to_string())?;
                }
            }
            _ = continue_stream.recv() => {
                log("Got continue signal")?;
                if let Err(e) = emit_status(&aqi_fether).await {
                    log(&e.to_string())?;
                }
            }
        }
    }
}
