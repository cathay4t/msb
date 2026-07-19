// SPDX-License-Identifier: Apache-2.0

use crate::{
    fs::{read_file, read_file_as_i64, read_file_as_u64},
    CliError, SwayBarBlock,
};

const SYSFS_BASE_DIR: &str = "/sys/class/power_supply/BAT0";

const EMOJI_CHARGING: &str = "⚡︎";
const EMOJI_BATTERY_GOOD: &str = "🔋";
const EMOJI_BATTERY_EMPTY: &str = "🪫";

pub(crate) fn get_battery() -> Result<Option<SwayBarBlock>, CliError> {
    if !std::fs::exists(SYSFS_BASE_DIR).unwrap_or_default() {
        return Ok(None);
    }

    let now = read_file_as_u64(&format!("{SYSFS_BASE_DIR}/energy_now"))?;
    let full =
        read_file_as_u64(&format!("{SYSFS_BASE_DIR}/energy_full_design"))?;
    let is_charging =
        read_file(&format!("{SYSFS_BASE_DIR}/status"))? != "Discharging";

    let percent = (now as f64 / full as f64 * 100.0) as u64;

    // Current power consumption in uW (always positive from sysfs)
    let consumption_abs =
        read_file_as_i64(&format!("{SYSFS_BASE_DIR}/power_now"))?;
    // Sign: discharge as negative, charge as positive
    let consumption_w = if is_charging {
        consumption_abs as f64 / 1_000_000.0
    } else {
        -(consumption_abs as f64) / 1_000_000.0
    };

    let time_left = if consumption_abs == 0 {
        f64::INFINITY
    } else if is_charging {
        if now >= full {
            0.0
        } else {
            (full - now) as f64 / consumption_abs as f64
        }
    } else {
        now as f64 / consumption_abs as f64
    };
    let time_left_str = if time_left.is_finite() {
        let h = time_left as u8;
        let m = (time_left.fract() * 60.0) as u8;
        format!("{h:02}:{m:02}")
    } else {
        "--:--".to_string()
    };

    let (charge_str, color) = if is_charging {
        (EMOJI_CHARGING, None)
    } else if !time_left.is_finite() || time_left < 0.5 {
        (EMOJI_BATTERY_EMPTY, Some(crate::COLOR_RED.to_string()))
    } else if percent > 40 {
        (EMOJI_BATTERY_GOOD, None)
    } else if percent > 20 {
        (EMOJI_BATTERY_GOOD, Some(crate::COLOR_YELLOW.to_string()))
    } else {
        (EMOJI_BATTERY_EMPTY, Some(crate::COLOR_RED.to_string()))
    };

    Ok(Some(SwayBarBlock {
        name: "battery".into(),
        color,
        full_text: format!(
            "{charge_str}: {percent}% {consumption_w:.1}W \
            {time_left_str}"
        ),
        min_width: Some(12),
        ..Default::default()
    }))
}
