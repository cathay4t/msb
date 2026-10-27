// SPDX-License-Identifier: Apache-2.0

mod aqi;
mod battery;
mod config;
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

fn iface_is_up(iface_name: &str) -> bool {
    std::fs::read_to_string(format!("/sys/class/net/{iface_name}/operstate"))
        .map(|state| state.trim() != "down")
        .unwrap_or(false)
}

async fn get_default_gateway_iface() -> Option<String> {
    let mut filter = nispor::NetStateFilter::minimum();
    filter.route = Some(nispor::NetStateRouteFilter::default());
    let state = nispor::NetState::retrieve_with_filter_async(&filter)
        .await
        .ok()?;
    state
        .routes
        .iter()
        .filter(|route| {
            route.gateway.is_some()
                && matches!(route.dst.as_deref(), None | Some("0.0.0.0/0"))
        })
        .filter(|route| route.oif.as_deref().map(iface_is_up).unwrap_or(false))
        .min_by_key(|route| {
            (
                route.address_family != nispor::AddressFamily::IPv4,
                route.metric.unwrap_or(u32::MAX),
            )
        })
        .and_then(|route| route.oif.clone())
}

fn get_time() -> SwayBarBlock {
    let now = chrono::offset::Local::now();
    SwayBarBlock {
        name: "time".to_string(),
        full_text: format!("{}", now.format("%m-%d %H:%M:%S")),
        ..Default::default()
    }
}

async fn emit_status(
    aqi_fether: Option<&crate::aqi::AqiFetcher>,
) -> Result<(), CliError> {
    let mut blocks: Vec<SwayBarBlock> = Vec::new();

    if let Some(b) = aqi_fether.and_then(|fetcher| fetcher.get()) {
        blocks.push(b);
    }
    let gateway_iface = get_default_gateway_iface().await;
    if let Some(ref iface_name) = gateway_iface {
        match crate::rate::get_rate(iface_name).await {
            Ok(b) => blocks.push(b),
            Err(e) => log::warn!("{e}"),
        }
        match crate::wifi::get_wifi(iface_name).await {
            Ok(b) => blocks.push(b),
            Err(e) => log::warn!("{e}"),
        }
    }
    match crate::cpu::get_cpu().await {
        Ok(b) => blocks.push(b),
        Err(e) => log::warn!("{e}"),
    }
    match crate::temp::get_temp() {
        Ok(bs) => blocks.extend_from_slice(&bs),
        Err(e) => log::warn!("{e}"),
    }
    match crate::sound::get_sound() {
        Ok(b) => blocks.push(b),
        Err(e) => log::warn!("{e}"),
    }
    match crate::battery::get_battery() {
        Ok(Some(b)) => blocks.push(b),
        Ok(None) => (),
        Err(e) => {
            log::warn!("{e}")
        }
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

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("debug"),
    )
    .init();

    let config = crate::config::Config::load()?;

    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(config.interval));

    println!("{}", serde_json::to_string(&SwayBarApiVersion::default())?);

    println!("[");

    let aqi_fether = match &config.aqi.location {
        Some(location) => Some(crate::aqi::AqiFetcher::new(location).await?),
        None => None,
    };

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = emit_status(aqi_fether.as_ref()).await {
                    log(&e.to_string())?;
                }
            }
            _ = continue_stream.recv() => {
                log("Got continue signal")?;
                if let Err(e) = emit_status(aqi_fether.as_ref()).await {
                    log(&e.to_string())?;
                }
            }
        }
    }
}
