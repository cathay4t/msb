// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Default)]
pub(crate) struct CliError {
    pub(crate) error_msg: String,
}

impl From<&str> for CliError {
    fn from(msg: &str) -> Self {
        Self {
            error_msg: msg.into(),
        }
    }
}

impl From<String> for CliError {
    fn from(error_msg: String) -> Self {
        Self { error_msg }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_msg)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            error_msg: format!("serde_json::Error: {e}"),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self {
            error_msg: format!("std::io::Error: {e}"),
        }
    }
}

impl From<nispor::NisporError> for CliError {
    fn from(e: nispor::NisporError) -> Self {
        Self {
            error_msg: format!("nispor::NisporError: {e}"),
        }
    }
}

impl From<std::string::FromUtf8Error> for CliError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self {
            error_msg: format!("std::string::FromUtf8Error: {e}"),
        }
    }
}

impl From<std::num::ParseIntError> for CliError {
    fn from(e: std::num::ParseIntError) -> Self {
        Self {
            error_msg: format!("std::num::ParseIntError: {e}"),
        }
    }
}

impl From<std::num::ParseFloatError> for CliError {
    fn from(e: std::num::ParseFloatError) -> Self {
        Self {
            error_msg: format!("std::num::ParseFloatError: {e}"),
        }
    }
}
