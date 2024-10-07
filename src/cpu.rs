// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crate::{fs::read_file, CliError, SwayBarBlock};

const INTERVAL: u64 = 500; // 0.5 second

#[derive(Debug)]
struct CpuUsageStat {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    soft_irq: u64,
    steal: u64,
}

impl From<&str> for CpuUsageStat {
    fn from(line: &str) -> Self {
        let mut cpu_stat_raw: Vec<u64> = line
            .split_ascii_whitespace()
            .map(|s| u64::from_str(s).unwrap_or_default())
            .collect();
        cpu_stat_raw.reverse();
        // The first entry is string `cpu`. Hence remove it.
        cpu_stat_raw.pop();
        CpuUsageStat {
            user: cpu_stat_raw.pop().unwrap_or_default(),
            nice: cpu_stat_raw.pop().unwrap_or_default(),
            system: cpu_stat_raw.pop().unwrap_or_default(),
            idle: cpu_stat_raw.pop().unwrap_or_default(),
            iowait: cpu_stat_raw.pop().unwrap_or_default(),
            irq: cpu_stat_raw.pop().unwrap_or_default(),
            soft_irq: cpu_stat_raw.pop().unwrap_or_default(),
            steal: cpu_stat_raw.pop().unwrap_or_default(),
            // kernel code `account_user_time()` indicate the user is already
            // containing guest time.
            // guest: cpu_stat_raw.pop().unwrap_or_default(),
            // guest_nice: cpu_stat_raw.pop().unwrap_or_default(),
        }
    }
}

impl CpuUsageStat {
    fn cpu_usage_percent(&self, old: &Self) -> u64 {
        (self.work_time() - old.work_time()) * 100
            / (self.total_time() - old.total_time())
    }

    fn work_time(&self) -> u64 {
        self.user
            .saturating_add(self.nice)
            .saturating_add(self.system)
            .saturating_add(self.irq)
            .saturating_add(self.soft_irq)
    }

    fn total_time(&self) -> u64 {
        self.work_time()
            .saturating_add(self.iowait)
            .saturating_add(self.idle)
            .saturating_add(self.steal)
    }

    fn retrieve() -> Result<Self, CliError> {
        let stat = read_file("/proc/stat")?;

        if let Some(line) = stat.split('\n').next() {
            Ok(CpuUsageStat::from(line))
        } else {
            Err("Failed to read first line of /proc/state".into())
        }
    }
}

pub(crate) async fn get_cpu() -> Result<SwayBarBlock, CliError> {
    let old_stat = CpuUsageStat::retrieve()?;
    tokio::time::sleep(std::time::Duration::from_millis(INTERVAL)).await;
    let new_stat = CpuUsageStat::retrieve()?;
    let percent = new_stat.cpu_usage_percent(&old_stat);
    let color = if percent >= 80 {
        Some(crate::COLOR_RED.to_string())
    } else if percent >= 50 {
        Some(crate::COLOR_YELLOW.to_string())
    } else {
        None
    };

    Ok(SwayBarBlock {
        name: "cpu".into(),
        full_text: format!("C: {percent:>3}%"),
        min_width: Some(7),
        color,
        ..Default::default()
    })
}
