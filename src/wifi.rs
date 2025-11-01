// SPDX-License-Identifier: Apache-2.0

use crate::{CliError, SwayBarBlock};

// Align with Microsoft `WLAN_ASSOCIATION_ATTRIBUTES`
const NOISE_FLOOR_DBM: i8 = -100;
const SIGNAL_MAX_DBM: i8 = -50;

// The clap is not stable feature yet.
#[allow(clippy::manual_clamp)]
fn dbm_to_percentage(dbm: i8) -> u8 {
    let dbm = if dbm > SIGNAL_MAX_DBM {
        SIGNAL_MAX_DBM
    } else if dbm < NOISE_FLOOR_DBM {
        NOISE_FLOOR_DBM
    } else {
        dbm
    };
    (100.0f64 * (NOISE_FLOOR_DBM - dbm) as f64
        / (NOISE_FLOOR_DBM - SIGNAL_MAX_DBM) as f64) as u8
}

pub(crate) async fn get_wifi(
    iface_name: &str,
) -> Result<SwayBarBlock, CliError> {
    let mut filter = nispor::NetStateFilter::minimum();
    let iface_filter = nispor::NetStateIfaceFilter::minimum();
    filter.iface = Some(iface_filter);
    let state = nispor::NetState::retrieve_with_filter_async(&filter).await?;
    if let Some(iface) = state.ifaces.get(iface_name) {
        if let Some(signal) = iface.wifi.as_ref().and_then(|w| w.signal) {
            let signal = dbm_to_percentage(signal);
            let color = if signal > 50 {
                None
            } else if signal > 25 {
                Some(crate::COLOR_YELLOW.to_string())
            } else {
                Some(crate::COLOR_RED.to_string())
            };
            return Ok(SwayBarBlock {
                name: "wifi".into(),
                full_text: format!("📶 {signal}%"),
                color,
                ..Default::default()
            });
        }
    }
    Ok(SwayBarBlock {
        name: "wifi".into(),
        full_text: "📶 0%".to_string(),
        color: Some(crate::COLOR_RED.to_string()),
        ..Default::default()
    })
}
