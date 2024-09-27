// SPDX-License-Identifier: Apache-2.0

use crate::{
    sysfs::{read_file_as_i64, read_file_as_u64},
    CliError, SwayBarBlock,
};

const SYSFS_BASE_DIR: &str = "/sys/class/power_supply/BAT0";

pub(crate) fn get_battery() -> Result<Option<SwayBarBlock>, CliError> {
    if !std::fs::exists(SYSFS_BASE_DIR).unwrap_or_default() {
        return Ok(None);
    }

    let now = read_file_as_u64(&format!("{SYSFS_BASE_DIR}/energy_now"))?;
    let full =
        read_file_as_u64(&format!("{SYSFS_BASE_DIR}/energy_full_design"))?;

    let percent = (now as f64 / full as f64 * 100.0) as u64;

    // Current power consumption in uW
    let consumption = read_file_as_i64(&format!("{SYSFS_BASE_DIR}/power_now"))?;
    let charge_str = if consumption > 0 { "🔋" } else { "⚡" };

    let time_left = if consumption > 0 {
        now as f64 / consumption as f64
    } else if now >= full {
        0.0
    } else {
        (full as f64 - now as f64) / (-consumption as f64)
    };
    let time_left_hour = time_left as u8;
    let time_left_min = (time_left.fract() * 60.0) as u8;

    let color = if time_left < 0.5 {
        crate::COLOR_RED.to_string()
    } else if percent > 60 {
        crate::COLOR_GREEN.to_string()
    } else if percent > 30 {
        crate::COLOR_YELLOW.to_string()
    } else {
        crate::COLOR_RED.to_string()
    };

    Ok(Some(SwayBarBlock {
        name: "battery".into(),
        color: Some(color),
        full_text: format!(
            "{charge_str}: {percent}% \
            {time_left_hour:02}:{time_left_min:02}"
        ),
        min_width: Some(12),
        ..Default::default()
    }))
}
