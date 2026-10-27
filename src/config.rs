// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use serde::Deserialize;

use crate::CliError;

const DEFAULT_INTERVAL: u64 = 10;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) interval: u64,
    pub(crate) aqi: AqiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval: DEFAULT_INTERVAL,
            aqi: AqiConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub(crate) struct AqiConfig {
    #[serde(default, deserialize_with = "deserialize_location")]
    pub(crate) location: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Location {
    Id(u32),
    Name(String),
}

fn deserialize_location<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(
        Option::<Location>::deserialize(deserializer)?.map(|location| {
            match location {
                Location::Id(id) => format!("@{id}"),
                Location::Name(name) => {
                    if name.parse::<u32>().is_ok() {
                        format!("@{name}")
                    } else {
                        name
                    }
                }
            }
        }),
    )
}

impl Config {
    pub(crate) fn load() -> Result<Self, CliError> {
        let home = std::env::var_os("HOME")
            .ok_or("HOME environment variable is not set")?;
        let path = Path::new(&home).join(".config/msb.yml");

        let config = match std::fs::read_to_string(&path) {
            Ok(content) => serde_yaml::from_str(&content).map_err(|e| {
                format!("Failed to parse config file {}: {e}", path.display())
            })?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Self::default()
            }
            Err(e) => {
                return Err(format!(
                    "Failed to read config file {}: {e}",
                    path.display()
                )
                .into());
            }
        };

        if config.interval == 0 {
            return Err(
                "Config option 'interval' must be greater than zero".into()
            );
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn location_id() {
        let config: Config =
            serde_yaml::from_str("aqi:\n  location: 1362\n").unwrap();
        assert_eq!(config.aqi.location.as_deref(), Some("@1362"));
    }

    #[test]
    fn location_id_as_string() {
        let config: Config =
            serde_yaml::from_str("aqi:\n  location: \"1362\"\n").unwrap();
        assert_eq!(config.aqi.location.as_deref(), Some("@1362"));
    }

    #[test]
    fn location_name() {
        let config: Config =
            serde_yaml::from_str("aqi:\n  location: chengdu\n").unwrap();
        assert_eq!(config.aqi.location.as_deref(), Some("chengdu"));
    }
}
