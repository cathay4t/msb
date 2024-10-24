// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::{
    fs::{read_dir, read_file, read_file_as_u64},
    CliError, SwayBarBlock,
};

const HWMON_NAMES: [&str; 3] = ["k10temp", "thinkpad", "coretemp"];

const SYSFS_HWMON_PATH: &str = "/sys/class/hwmon/";

pub(crate) fn get_temp() -> Result<SwayBarBlock, CliError> {
    let mut degree = 0u64;

    let mut name_to_path: HashMap<String, String> = HashMap::new();

    for subdir in read_dir(SYSFS_HWMON_PATH)? {
        let subdir = format!("{SYSFS_HWMON_PATH}/{subdir}");

        let hwmon_name = read_file(&format!("{subdir}/name"))?;
        name_to_path.insert(hwmon_name, subdir);
    }

    for prefered in &HWMON_NAMES {
        if let Some(hwmon_dir) = name_to_path.get(prefered.to_string().as_str()) {
            let temp_file_path = format!("{hwmon_dir}/temp1_input");
            if std::path::Path::new(&temp_file_path).is_file() {
                degree = read_file_as_u64(&temp_file_path)? / 1000;
                break;
            }
        }
    }
    let color = if degree >= 80 {
        Some(crate::COLOR_RED.to_string())
    } else {
        None
    };

    Ok(SwayBarBlock {
        name: "temp".into(),
        full_text: format!("🌡: {degree:>2}°C"),
        min_width: Some(8),
        color,
        ..Default::default()
    })
}
