// SPDX-License-Identifier: Apache-2.0

use crate::{sysfs::read_file_as_u64, CliError, SwayBarBlock};

const INTERVAL: u64 = 500;

const KIB: u64 = 1 << 10;
const MIB: u64 = 1 << 20;
const GIB: u64 = 1 << 30;
const TIB: u64 = 1 << 40;

fn bytes_to_human(bytes: u64) -> String {
    if bytes > TIB {
        format!("{}.{} TiB", bytes / TIB, bytes % TIB / GIB)
    } else if bytes >= GIB {
        format!("{}.{} GiB", bytes / GIB, bytes % GIB / MIB)
    } else if bytes >= MIB {
        format!("{}.{} MiB", bytes / MIB, bytes % MIB / KIB)
    } else if bytes >= KIB {
        format!("{}.{} KiB", bytes / KIB, bytes % KIB)
    } else {
        format!("{bytes} B")
    }
}

pub(crate) async fn get_rate(
    iface_name: &str,
) -> Result<SwayBarBlock, CliError> {
    let (rx_speed, tx_speed) = get_net_speed(iface_name).await?;
    Ok(SwayBarBlock {
        name: "rate".into(),
        full_text: format!(
            "{iface_name:>8}: v {: >9}/s ^ {: >9}/s",
            rx_speed, tx_speed
        ),
        min_width: Some(28),
        ..Default::default()
    })
}

async fn get_net_speed(iface_name: &str) -> Result<(String, String), CliError> {
    let (cur_rx, cur_tx) = get_net_bytes(iface_name)?;
    tokio::time::sleep(std::time::Duration::from_millis(INTERVAL)).await;
    let (new_rx, new_tx) = get_net_bytes(iface_name)?;
    let rx_speed = (new_rx - cur_rx) * 1000 / INTERVAL;
    let tx_speed = (new_tx - cur_tx) * 1000 / INTERVAL;
    Ok((bytes_to_human(rx_speed), bytes_to_human(tx_speed)))
}

fn get_net_bytes(iface_name: &str) -> Result<(u64, u64), CliError> {
    let rx_file = format!("/sys/class/net/{}/statistics/rx_bytes", iface_name);
    let tx_file = format!("/sys/class/net/{}/statistics/tx_bytes", iface_name);
    if std::path::Path::new(&rx_file).exists() {
        Ok((read_file_as_u64(&rx_file)?, read_file_as_u64(&tx_file)?))
    } else {
        Err(format!("{rx_file} does not exist").into())
    }
}
