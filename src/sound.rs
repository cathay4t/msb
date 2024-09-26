// SPDX-License-Identifier: Apache-2.0

use std::process::Command;
use std::str::FromStr;

use crate::{CliError, SwayBarBlock};

pub(crate) fn get_sound() -> Result<SwayBarBlock, CliError> {
    let output: String = String::from_utf8(
        Command::new("wpctl")
            .arg("get-volume")
            .arg("@DEFAULT_AUDIO_SINK@")
            .output()?
            .stdout,
    )?;
    let output = output.trim();
    if let Some(vol) = output.strip_prefix("Volume: ") {
        let (vol, is_muted) = if let Some(vol) = vol.strip_suffix(" [MUTED]") {
            ((f32::from_str(vol)? * 100.0) as u32, true)
        } else {
            ((f32::from_str(vol)? * 100.0) as u32, false)
        };
        let color = if is_muted {
            Some(crate::COLOR_YELLOW.to_string())
        } else {
            None
        };

        let full_text = if is_muted {
            format!("🔇: {vol}%")
        } else {
            format!("♪: {vol}%")
        };

        Ok(SwayBarBlock {
            name: "sound".into(),
            full_text,
            color,
            ..Default::default()
        })
    } else {
        Err(format!("Failed to extract sound volume: {output}").into())
    }
}
