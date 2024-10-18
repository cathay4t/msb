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

    // Current power consumption in uW
    let consumption = read_file_as_i64(&format!("{SYSFS_BASE_DIR}/power_now"))?;

    let time_left = if is_charging {
        (full - now) as f64 / consumption as f64
    } else if now >= full {
        0.0
    } else {
        now as f64 / consumption as f64
    };
    let time_left_hour = time_left as u8;
    let time_left_min = (time_left.fract() * 60.0) as u8;

    let (charge_str, color) = if is_charging {
        (EMOJI_CHARGING, None)
    } else if time_left < 0.5 {
        (EMOJI_BATTERY_EMPTY, Some(crate::COLOR_RED.to_string()))
    } else if percent > 60 {
        (EMOJI_BATTERY_GOOD, None)
    } else if percent > 30 {
        (EMOJI_BATTERY_GOOD, Some(crate::COLOR_YELLOW.to_string()))
    } else {
        (EMOJI_BATTERY_EMPTY, Some(crate::COLOR_RED.to_string()))
    };

    Ok(Some(SwayBarBlock {
        name: "battery".into(),
        color,
        full_text: format!(
            "{charge_str}: {percent}% \
            {time_left_hour:02}:{time_left_min:02}"
        ),
        min_width: Some(12),
        ..Default::default()
    }))
}
