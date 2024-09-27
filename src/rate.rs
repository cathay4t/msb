// SPDX-License-Identifier: Apache-2.0

use std::io::Read;

use crate::{CliError, SwayBarBlock};

const INTERVAL: u64 = 500;

pub(crate) async fn get_rate(
    iface_name: &str,
) -> Result<SwayBarBlock, CliError> {
    let (rx_speed, tx_speed) = get_net_speed(iface_name).await?;
    Ok(SwayBarBlock {
        name: "rate".into(),
        full_text: format!(
            "{iface_name:>8}: v{: >9}/s ^{: >9}/s",
            rx_speed, tx_speed
        ),
        min_width: Some(26),
        ..Default::default()
    })
}

fn read_file(file_path: &str) -> Result<String, CliError> {
    let mut fd = std::fs::File::open(file_path)?;
    let mut contents = String::new();
    fd.read_to_string(&mut contents)?;
    Ok(contents)
}

fn read_sysfs_as_u64(file_path: &str) -> Result<u64, CliError> {
    let content = read_file(file_path)?;
    Ok(content.trim().parse::<u64>()?)
}

async fn get_net_speed(iface_name: &str) -> Result<(String, String), CliError> {
    let (cur_rx, cur_tx) = get_net_bytes(iface_name)?;
    tokio::time::sleep(std::time::Duration::from_millis(INTERVAL)).await;
    let (new_rx, new_tx) = get_net_bytes(iface_name)?;
    let rx_speed = (new_rx - cur_rx) * 1000 / INTERVAL;
    let tx_speed = (new_tx - cur_tx) * 1000 / INTERVAL;
    Ok((
        bytesize::ByteSize::b(rx_speed).to_string_as(true),
        bytesize::ByteSize::b(tx_speed).to_string_as(true),
    ))
}

fn get_net_bytes(iface_name: &str) -> Result<(u64, u64), CliError> {
    let rx_file = format!("/sys/class/net/{}/statistics/rx_bytes", iface_name);
    let tx_file = format!("/sys/class/net/{}/statistics/tx_bytes", iface_name);
    if std::path::Path::new(&rx_file).exists() {
        Ok((read_sysfs_as_u64(&rx_file)?, read_sysfs_as_u64(&tx_file)?))
    } else {
        Err(format!("{rx_file} does not exist").into())
    }
}
